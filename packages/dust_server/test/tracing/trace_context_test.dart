import 'dart:math';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

/// `traceparent` is a wire format shared with services written in something
/// else, so the parsing has to match the specification rather than be merely
/// self-consistent. A field read one character wrong joins the wrong trace, or
/// silently starts a new one, and nothing fails loudly.

void main() {
  const traceId = '4bf92f3577b34da6a3ce929d0e0e4736';
  const spanId = '00f067aa0ba902b7';

  group('parsing', () {
    test('reads a sampled header', () {
      final context = TraceContext.parse('00-$traceId-$spanId-01')!;

      expect(
        [context.traceId, context.spanId, context.sampled],
        [traceId, spanId, true],
      );
    });

    test('reads an unsampled header', () {
      expect(TraceContext.parse('00-$traceId-$spanId-00')!.sampled, isFalse);
    });

    test('reads only the sampled bit out of the flags', () {
      // Other flag bits are reserved; a future one must not disable sampling.
      expect(TraceContext.parse('00-$traceId-$spanId-ff')!.sampled, isTrue);
      expect(TraceContext.parse('00-$traceId-$spanId-fe')!.sampled, isFalse);
    });

    test('ignores surrounding whitespace', () {
      expect(TraceContext.parse('  00-$traceId-$spanId-01  '), isNotNull);
    });

    test('accepts trailing sections a later version may add', () {
      expect(TraceContext.parse('00-$traceId-$spanId-01-extra'), isNotNull);
    });

    test('is null for a header that is absent', () {
      expect(TraceContext.parse(null), isNull);
    });

    test('is null for an unknown version', () {
      expect(TraceContext.parse('01-$traceId-$spanId-01'), isNull);
    });

    test('is null when a section is missing', () {
      expect(TraceContext.parse('00-$traceId-$spanId'), isNull);
    });

    test('is null for a trace id of the wrong length', () {
      expect(TraceContext.parse('00-abc-$spanId-01'), isNull);
    });

    test('is null for a span id of the wrong length', () {
      expect(TraceContext.parse('00-$traceId-abc-01'), isNull);
    });

    test('is null for upper-case hex, which the format forbids', () {
      expect(
        TraceContext.parse('00-${traceId.toUpperCase()}-$spanId-01'),
        isNull,
      );
    });

    test('is null for a non-hex character', () {
      expect(TraceContext.parse('00-${'z' * 32}-$spanId-01'), isNull);
    });

    test('is null for the reserved all-zero trace id', () {
      expect(TraceContext.parse('00-${'0' * 32}-$spanId-01'), isNull);
    });

    test('is null for the reserved all-zero span id', () {
      expect(TraceContext.parse('00-$traceId-${'0' * 16}-01'), isNull);
    });
  });

  group('formatting', () {
    test('round-trips through parse', () {
      const header = '00-$traceId-$spanId-01';

      expect(TraceContext.parse(header)!.traceparent, header);
    });

    test('writes the unsampled flag as 00', () {
      const context = TraceContext(
        traceId: traceId,
        spanId: spanId,
        sampled: false,
      );

      expect(context.traceparent, endsWith('-00'));
    });

    test('describes itself as the header', () {
      expect(TraceContext.parse('00-$traceId-$spanId-01').toString(),
          '00-$traceId-$spanId-01');
    });
  });

  group('starting a trace', () {
    test('produces ids of the right shape', () {
      final context = TraceContext.start(random: Random(1));

      expect(TraceContext.parse(context.traceparent), isNotNull);
    });

    test('produces a different trace each time', () {
      final first = TraceContext.start();
      final second = TraceContext.start();

      expect(first.traceId, isNot(second.traceId));
    });

    test('can start unsampled', () {
      expect(TraceContext.start(sampled: false).sampled, isFalse);
    });
  });

  group('a child', () {
    test('stays in the same trace', () {
      final parent = TraceContext.parse('00-$traceId-$spanId-01')!;

      expect(parent.child().traceId, parent.traceId);
    });

    test('takes a new span id', () {
      final parent = TraceContext.parse('00-$traceId-$spanId-01')!;

      expect(parent.child().spanId, isNot(parent.spanId));
    });

    test('inherits the sampling decision', () {
      final parent = TraceContext.parse('00-$traceId-$spanId-00')!;

      expect(parent.child().sampled, isFalse);
    });
  });
}
