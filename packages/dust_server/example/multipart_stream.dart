import 'dart:io';
import 'dart:math';

import 'package:dust_server/server.dart';

/// Accepting an upload larger than memory.
///
/// `multipart()` buffers the whole body so several parts can be read in any
/// order. That is right for a form and wrong for a video: a 2 GB upload is 2 GB
/// of heap, and a handful of concurrent ones is an outage.
///
/// `multipartStream()` hands the parts over one at a time, so each can be piped
/// straight to disk and never held. The cost is the trade you are making
/// knowingly: the parts arrive **in order** and are **consumable once**. A
/// handler that must see part three before part one has to buffer, and should
/// use `multipart()`.
///
/// > **The filename is client input.** `../../etc/passwd` is a perfectly valid
/// > filename as far as the client is concerned, so it is fine to record and
/// > unsafe to join onto a path. Store under an identifier you generated, as
/// > here, and keep the original name as data.
///
/// Two limits, because they stop different things:
///
/// * the **body** limit bounds the whole request, and is checked up front when
///   the client declared a `content-length` and as the bytes flow when it did
///   not — a streamed upload usually has no length to check;
/// * the **part** limit bounds one file. Without it, one part can be the whole
///   body limit, and an upload with no ceiling is a way to fill your disk.
///
/// Run it with `dart run example/multipart_stream.dart`:
///
/// ```bash
/// head -c 5000000 /dev/urandom > big.bin
/// curl -s -X POST localhost:8080/upload -F 'caption=a big one' -F 'file=@big.bin'
/// curl -si -X POST localhost:8080/upload -F 'file=@big.bin' -F 'file2=@big.bin' \
///   -F 'file3=@big.bin'   # 413 once the total passes the body limit
/// ```
Future<void> main() async {
  final uploads = await Directory.systemTemp.createTemp('dust-uploads-');
  final server = await serve(
    buildApp(uploads.path),
    InternetAddress.anyIPv4,
    8080,
  );
  stdout.writeln('writing uploads to ${uploads.path}');
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp(String directory) {
  return Router()
    ..route('/upload', post(upload, status: 201))
    ..withState(UploadDirectory(directory));
}

/// `POST /upload`
Future<Map<String, Object?>> upload(Request request) async {
  final target = await request.state<UploadDirectory>();
  // 10 MB across the whole request, whatever it is made of.
  final body = await request.multipartStream(limit: 10 * 1024 * 1024);

  final stored = <Map<String, Object?>>[];
  final fields = <String, String>{};

  await body.forEachPart((part) async {
    if (!part.isFile) {
      // Small by definition, so collecting it is fine — with a limit anyway.
      fields[part.name] = await part.readText(limit: 4096);
      return;
    }

    // An id we generated, not the name the client sent. The original is kept
    // as data, where it cannot become a path.
    final id = target.newId();
    final file = File('${target.path}/$id');
    final sink = file.openWrite();
    try {
      final bytes = await part.writeTo(sink, limit: 5 * 1024 * 1024);
      stored.add({
        'id': id,
        'filename': part.filename,
        'contentType': part.contentType,
        'bytes': bytes,
      });
    } finally {
      // Closed on the failure path too: a rejected upload must not leak a
      // handle, and the partial file is cleaned up by the caller's retry.
      await sink.close();
    }
  });

  return {'fields': fields, 'files': stored};
}

/// Where uploads land, and how they are named.
final class UploadDirectory {
  /// Writes into [path].
  UploadDirectory(this.path, {Random? random})
      : _random = random ?? Random.secure();

  /// The directory.
  final String path;

  final Random _random;

  /// An identifier nobody can guess or walk.
  String newId() {
    const alphabet = 'abcdefghijklmnopqrstuvwxyz0123456789';
    return [
      for (var index = 0; index < 16; index++)
        alphabet[_random.nextInt(alphabet.length)],
    ].join();
  }
}
