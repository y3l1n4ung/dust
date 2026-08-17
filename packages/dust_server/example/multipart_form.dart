import 'dart:io';

import 'package:dust_server/server.dart';

/// Handling a file upload.
///
/// `multipart()` hands back a [MultipartForm]: `file(name)` for the parts that
/// carried a filename, `field<T>` for the ordinary ones. Both return a
/// `Result`, so a missing part is a value you handle rather than a throw.
///
/// > **This buffers.** The whole body is read into memory before the handler
/// > runs, capped by the extractor's `limit`, which answers **413** when
/// > exceeded. That is fine for an avatar and wrong for a video: streaming
/// > multipart straight to disk is a documented gap in the runtime. Set the
/// > limit to what you actually accept rather than leaving the default.
///
/// Run it with `dart run example/multipart_form.dart`:
///
/// ```bash
/// curl -s -X POST localhost:8080/upload \
///   -F 'caption=my cat' -F 'photo=@README.md'
/// curl -i -X POST localhost:8080/upload -F 'caption=no file'   # 422
/// curl -i -X POST localhost:8080/upload -d 'caption=x'         # 415
/// ```
Future<void> main() async {
  final server = await serveRouter(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() => Router()..route('/upload', post(upload));

/// `POST /upload`
Future<Result<Map<String, Object?>, Rejection>> upload(Request request) async {
  final form = await request.multipart();

  switch (form.file('photo')) {
    case Err(:final error):
      return Err(error);
    case Ok(value: final file):
      final caption = switch (form.field<String?>('caption')) {
        Ok(:final value) => value,
        Err() => null,
      };

      // The filename is client-supplied text. Fine to record, unsafe to join
      // onto a path: `../../etc/passwd` is a valid filename as far as the
      // client is concerned. Store under an id you generated instead.
      return Ok({
        'filename': file.filename,
        'bytes': file.bytes.length,
        'caption': caption,
      });
  }
}
