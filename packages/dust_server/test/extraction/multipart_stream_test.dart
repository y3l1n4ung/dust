import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

/// Streaming multipart: the parts arrive one at a time and are never all held
/// in memory at once, which is what makes an upload larger than memory possible.

const _boundary = 'X-DUST';

/// Builds a multipart body from [parts], each `(name, filename, content)`.
List<int> _body(List<(String, String?, List<int>)> parts) {
  final out = <int>[];
  for (final (name, filename, content) in parts) {
    final disposition = filename == null
        ? 'form-data; name="$name"'
        : 'form-data; name="$name"; filename="$filename"';
    out
      ..addAll(utf8.encode('--$_boundary\r\n'))
      ..addAll(utf8.encode('content-disposition: $disposition\r\n\r\n'))
      ..addAll(content)
      ..addAll(utf8.encode('\r\n'));
  }
  out.addAll(utf8.encode('--$_boundary--\r\n'));
  return out;
}

/// A request carrying [body], optionally in several chunks.
Request _request(List<int> body, {int chunk = 1 << 20, int? contentLength}) {
  final chunks = <List<int>>[];
  for (var index = 0; index < body.length; index += chunk) {
    chunks.add(
      body.sublist(index, (index + chunk).clamp(0, body.length)),
    );
  }

  return Request(
    'POST',
    Uri.parse('http://localhost/upload'),
    headers: {
      'content-type': 'multipart/form-data; boundary=$_boundary',
      if (contentLength != null) 'content-length': '$contentLength',
    },
    body: Stream<List<int>>.fromIterable(chunks),
  );
}

Future<StreamedMultipart> _extract(Request request) async =>
    expectOk(await const StreamedMultipartExtractable().extract(request));

void main() {
  group('the streaming multipart extractor', () {
    test('reads a field and a file in order', () async {
      final body = await _extract(
        _request(
          _body([
            ('caption', null, utf8.encode('my cat')),
            ('photo', 'cat.txt', utf8.encode('meow')),
          ]),
        ),
      );

      final seen = <String>[];
      await body.forEachPart((part) async {
        seen.add('${part.name}:${part.filename}:${await part.readText()}');
      });

      expect(seen, ['caption:null:my cat', 'photo:cat.txt:meow']);
    });

    test('isFile distinguishes a file from an ordinary field', () async {
      final body = await _extract(
        _request(
          _body([
            ('caption', null, utf8.encode('text')),
            ('photo', 'cat.png', utf8.encode('bytes')),
          ]),
        ),
      );

      final files = <String>[];
      await body.forEachPart((part) async {
        if (part.isFile) files.add(part.name);
      });

      expect(files, ['photo']);
    });

    test('writes a part out without collecting it', () async {
      final target = File(
        '${Directory.systemTemp.createTempSync('dust-mp-').path}/out.bin',
      );
      addTearDown(() => target.parent.delete(recursive: true));

      final body = await _extract(
        _request(_body([('photo', 'cat.bin', List.filled(200000, 7))])),
      );

      var written = 0;
      await body.forEachPart((part) async {
        final sink = target.openWrite();
        written = await part.writeTo(sink, limit: 1 << 20);
        await sink.close();
      });

      expect(written, 200000);
      expect(target.lengthSync(), 200000);
    });

    test('never holds the whole body at once', () async {
      // The property the whole extractor exists for. Five parts of 200 KB each,
      // and the largest chunk handed over is one socket chunk, not a megabyte.
      final body = await _extract(
        _request(
          _body([
            for (var index = 0; index < 5; index++)
              ('file$index', 'f$index.bin', List.filled(200000, 1)),
          ]),
          chunk: 8192,
        ),
      );

      var largestChunk = 0;
      var total = 0;
      await body.forEachPart((part) async {
        await for (final piece in part.content) {
          largestChunk =
              piece.length > largestChunk ? piece.length : largestChunk;
          total += piece.length;
        }
      });

      expect(total, 1000000);
      expect(largestChunk, lessThanOrEqualTo(8192));
    });

    test('forEachPart drains a part the callback ignored', () async {
      // Forgetting to drain stalls the request, so the convenience does it.
      final body = await _extract(
        _request(
          _body([
            ('skipped', 'a.bin', List.filled(50000, 2)),
            ('wanted', null, utf8.encode('here')),
          ]),
        ),
      );

      final seen = <String>[];
      await body.forEachPart((part) async {
        if (part.name == 'wanted') seen.add(await part.readText());
      });

      expect(seen, ['here']);
    });

    test('isTaken reports whether anything read the part', () async {
      final body = await _extract(
        _request(_body([('a', null, utf8.encode('one'))])),
      );

      await for (final part in body.parts) {
        expect(part.isTaken, isFalse);
        await part.readBytes();
        expect(part.isTaken, isTrue);
      }
    });

    test('skip on a part already read does nothing rather than throwing',
        () async {
      final body = await _extract(
        _request(_body([('a', null, utf8.encode('one'))])),
      );

      await for (final part in body.parts) {
        await part.readBytes();
        await part.skip();
      }
    });

    test('skip lets the next part start', () async {
      final body = await _extract(
        _request(
          _body([
            ('first', 'a.bin', List.filled(50000, 3)),
            ('second', null, utf8.encode('second')),
          ]),
        ),
      );

      final seen = <String>[];
      await for (final part in body.parts) {
        if (part.name == 'first') {
          await part.skip();
        } else {
          seen.add(await part.readText());
        }
      }

      expect(seen, ['second']);
    });
  });
}
