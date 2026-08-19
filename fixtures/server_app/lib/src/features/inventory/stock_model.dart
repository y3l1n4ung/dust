import 'package:dust_dart/db.dart';
import 'package:dust_dart/serde.dart';

part 'stock_model.g.dart';

/// What is left of one item.
@Derive([ToString(), Eq(), Serialize(), FromRow()])
final class Stock with _$Stock {
  /// Creates a [Stock].
  const Stock({required this.item, required this.onHand});

  /// What it is.
  final String item;

  /// How many are left.
  @Sqlx(rename: 'on_hand')
  final int onHand;
}
