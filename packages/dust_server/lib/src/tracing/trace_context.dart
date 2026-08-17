import 'dart:math';

/// The identifiers that tie one request's spans to the work it caused.
///
/// The wire format is W3C Trace Context, which is what makes a trace survive a
/// hop into a service written in something else: `traceparent` is
/// `00-<32 hex trace id>-<16 hex span id>-<2 hex flags>`. Inventing a format
/// here would mean a trace that stops at the first boundary.
final class TraceContext {
  /// Creates a context from its parts.
  const TraceContext({
    required this.traceId,
    required this.spanId,
    required this.sampled,
  });

  /// Starts a new trace.
  factory TraceContext.start({Random? random, bool sampled = true}) {
    final source = random ?? _random;
    return TraceContext(
      traceId: _hex(source, 16),
      spanId: _hex(source, 8),
      sampled: sampled,
    );
  }

  /// Reads a `traceparent` header, or `null` when it is absent or malformed.
  ///
  /// A malformed header is dropped rather than rejected: an unreadable trace
  /// id is an observability problem, and failing the request over it would
  /// turn a broken collector into an outage.
  static TraceContext? parse(String? traceparent) {
    if (traceparent == null) return null;

    final parts = traceparent.trim().split('-');
    if (parts.length < 4) return null;
    if (parts[0] != '00') return null;
    if (!_isHex(parts[1], 32) || !_isHex(parts[2], 16)) return null;
    if (!_isHex(parts[3], 2)) return null;

    // All-zero ids are reserved as "invalid" by the specification.
    if (parts[1] == '0' * 32 || parts[2] == '0' * 16) return null;

    return TraceContext(
      traceId: parts[1],
      spanId: parts[2],
      sampled: int.parse(parts[3], radix: 16) & 0x01 == 0x01,
    );
  }

  /// The trace this belongs to, 32 hex characters.
  final String traceId;

  /// This span, 16 hex characters.
  final String spanId;

  /// Whether the trace was sampled, which downstream services honour.
  final bool sampled;

  /// A child of this context: same trace, a new span.
  TraceContext child({Random? random}) => TraceContext(
        traceId: traceId,
        spanId: _hex(random ?? _random, 8),
        sampled: sampled,
      );

  /// The `traceparent` header value for this context.
  String get traceparent => '00-$traceId-$spanId-${sampled ? '01' : '00'}';

  @override
  String toString() => traceparent;

  static final _random = Random.secure();

  static String _hex(Random random, int bytes) {
    final buffer = StringBuffer();
    for (var index = 0; index < bytes; index++) {
      buffer.write(random.nextInt(256).toRadixString(16).padLeft(2, '0'));
    }
    return buffer.toString();
  }

  static bool _isHex(String value, int length) {
    if (value.length != length) return false;
    for (final unit in value.codeUnits) {
      final isDigit = unit >= 0x30 && unit <= 0x39;
      final isLower = unit >= 0x61 && unit <= 0x66;
      if (!isDigit && !isLower) return false;
    }
    return true;
  }
}
