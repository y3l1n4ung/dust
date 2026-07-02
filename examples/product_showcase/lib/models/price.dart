import 'package:dust_dart/derive.dart';

part 'price.g.dart';

/// Price model for the product showcase example.
@Derive([ToString(), Eq(), CopyWith()])
class Price with _$Price {
  /// Creates a [Price].
  const Price({
    required this.currency,
    required this.cents,
    required this.tags,
  });

  /// Currency.
  final String currency;

  /// Cents.
  final int cents;

  /// Tags.
  final List<String> tags;
}
