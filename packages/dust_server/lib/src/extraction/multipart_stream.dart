import 'dart:async';
import 'dart:convert';
import 'dart:io' show HeaderValue;
import 'dart:typed_data';

import 'package:dust_dart/fp.dart';
import 'package:mime/mime.dart';
import 'package:shelf/shelf.dart';

import '../request/request_parts.dart';
import '../response/rejection.dart';
import 'body_reader.dart';
import 'extractable.dart';

/// One part of a multipart body, still arriving.
///
/// [content] is the part's bytes as they come off the socket. Read it, or the
/// next part cannot start — the parts share one connection, and skipping ahead
/// without draining stalls the stream. [skip] is how you say you do not want it.
final class MultipartPart {
  /// Wraps one streaming part.
  MultipartPart({
    required this.name,
    required Stream<List<int>> content,
    this.filename,
    this.contentType,
  }) : _content = content;

  final Stream<List<int>> _content;
  bool _taken = false;

  /// The form field name.
  final String name;

  /// The client-supplied file name, when the part was a file.
  ///
  /// Client input. Fine to record, unsafe to join onto a path:
  /// `../../etc/passwd` is a valid filename as far as the client is concerned.
  /// Store under an identifier you generated.
  final String? filename;

  /// The part's own `content-type`, when it declared one.
  final String? contentType;

  /// The part's bytes, consumable once.
  ///
  /// Reading it twice throws: the bytes came off a socket and were not kept.
  Stream<List<int>> get content {
    _taken = true;
    return _content;
  }

  /// Whether anything has started reading this part.
  bool get isTaken => _taken;

  /// Whether the part was a file rather than an ordinary field.
  bool get isFile => filename != null;

  /// Collects the part into memory.
  ///
  /// Fine for a form field, and the thing streaming exists to avoid for a file.
  /// [limit] bounds it anyway, because "this one is small" is a claim the client
  /// is making.
  Future<Uint8List> readBytes({int limit = defaultBodyLimit}) async {
    final builder = BytesBuilder(copy: false);
    await for (final chunk in content) {
      builder.add(chunk);
      if (builder.length > limit) {
        throw Rejection.payloadTooLarge(
          'multipart part "$name" exceeds $limit bytes',
        );
      }
    }
    return builder.takeBytes();
  }

  /// Collects the part as UTF-8 text.
  Future<String> readText({int limit = defaultBodyLimit}) async =>
      utf8.decode(await readBytes(limit: limit));

  /// Writes the part into [sink] without holding it in memory.
  ///
  /// Returns how many bytes were written, and refuses past [limit] — an upload
  /// with no ceiling is a way to fill your disk.
  ///
  /// The sink is not closed: the caller opened it and may have more to write.
  Future<int> writeTo(
    StreamConsumer<List<int>> sink, {
    int limit = defaultBodyLimit,
  }) async {
    var written = 0;
    final counted = content.map((chunk) {
      written += chunk.length;
      if (written > limit) {
        throw Rejection.payloadTooLarge(
          'multipart part "$name" exceeds $limit bytes',
        );
      }
      return chunk;
    });

    await sink.addStream(counted);
    return written;
  }

  /// Discards the part, so the next one can start.
  ///
  /// Does nothing when something already read it — draining a consumed stream
  /// would throw, and the bytes are gone either way.
  Future<void> skip() async {
    if (_taken) return;
    await content.drain<void>();
  }
}

/// A multipart body, delivered a part at a time.
final class StreamedMultipart {
  /// Wraps the [parts] stream.
  const StreamedMultipart(this.parts);

  /// The parts, in the order the client sent them.
  ///
  /// Consumable once, and **in order**: each part has to be read or skipped
  /// before the next arrives, because they share one connection. Collecting the
  /// stream into a list first and reading the parts afterwards will hang.
  final Stream<MultipartPart> parts;

  /// Runs [onPart] for each part, discarding any it did not read.
  ///
  /// The convenience worth having: a part nobody reads stalls the request,
  /// because the next one cannot start until this one is off the connection.
  /// This makes forgetting impossible.
  Future<void> forEachPart(
    FutureOr<void> Function(MultipartPart part) onPart,
  ) async {
    await for (final part in parts) {
      await onPart(part);
      // Only when the callback left it alone; draining a consumed stream throws.
      await part.skip();
    }
  }
}

/// Hands a `multipart/form-data` body to the handler a part at a time.
///
/// The difference from `MultipartExtractable`, which buffers the whole body:
/// this never holds more than one chunk, so an upload can be written straight to
/// disk or to object storage. That is what makes a file larger than memory
/// possible at all.
///
/// The cost is that the parts are ordered and consumable once. A handler that
/// needs to look at part three before part one has to buffer, and should use the
/// buffering extractor instead.
///
/// ```dart
/// Future<Map<String, Object?>> upload(Request request) async {
///   final body = await request.extract(const StreamedMultipartExtractable());
///   var bytes = 0;
///
///   await body.forEachPart((part) async {
///     if (!part.isFile) return;
///     final file = File('uploads/${newId()}').openWrite();
///     bytes = await part.writeTo(file, limit: 50 * 1024 * 1024);
///     await file.close();
///   });
///
///   return {'bytes': bytes};
/// }
/// ```
///
/// [limit] bounds the **whole body**, and it is enforced as the bytes flow
/// rather than up front: a streamed upload usually arrives without a
/// `content-length`, so there is nothing to check before reading. A part that
/// pushes the total past it fails with 413 mid-stream, which is the earliest
/// anything can.
final class StreamedMultipartExtractable
    implements FromRequest<StreamedMultipart> {
  /// Streams the multipart body, capped at [limit] bytes in total.
  const StreamedMultipartExtractable({this.limit = 64 * 1024 * 1024});

  /// The maximum total body size, in bytes.
  final int limit;

  @override
  Future<Result<StreamedMultipart, Rejection>> extract(Request request) async {
    final parts = RequestParts.of(request);
    if (parts.mediaType != 'multipart/form-data') {
      return const Err(
        Rejection.unsupportedMediaType('expected multipart/form-data'),
      );
    }

    final boundary = parts.contentType?.parameters['boundary'];
    if (boundary == null) {
      return const Err(
        Rejection.badRequest('multipart body declares no boundary'),
      );
    }

    final configured = request.context[bodyLimitContextKey];
    final effective = configured is int ? configured : limit;

    // Checked up front when the client declared one, so an oversized upload is
    // refused before a byte of it is read.
    final declared = parts.contentLength;
    if (declared != null && declared > effective) {
      return Err(_tooLarge(effective));
    }

    return Ok(
      StreamedMultipart(
        _partsOf(_capped(request.read(), effective), boundary),
      ),
    );
  }

  /// Fails the stream once more than [limit] bytes have gone past.
  Stream<List<int>> _capped(Stream<List<int>> body, int limit) {
    var seen = 0;
    return body.map((chunk) {
      seen += chunk.length;
      if (seen > limit) throw _tooLarge(limit);
      return chunk;
    });
  }

  Stream<MultipartPart> _partsOf(Stream<List<int>> body, String boundary) {
    return body.transform(MimeMultipartTransformer(boundary)).map((part) {
      final disposition =
          HeaderValue.parse(part.headers['content-disposition'] ?? '');

      return MultipartPart(
        name: disposition.parameters['name'] ?? '',
        content: part,
        filename: disposition.parameters['filename'],
        contentType: part.headers['content-type'],
      );
    });
  }

  static Rejection _tooLarge(int limit) =>
      Rejection.payloadTooLarge('body exceeds $limit bytes');
}
