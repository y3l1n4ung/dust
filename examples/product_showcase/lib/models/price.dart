import 'package:derive_annotation/derive_annotation.dart';

part 'price.g.dart';

@Derive([Debug(), Clone(), PartialEq(), Hash(), CopyWith()])
class Price with _$PriceDust {
  const Price({
    required this.currency,
    required this.cents,
    required this.tags,
  });

  final String currency;
  final int cents;
  final List<String> tags;
}
