import 'package:dust_dart/derive.dart';

part 'nested_bundle.g.dart';

/// Nested bundle model for the product showcase example.
@Derive([ToString(), Eq(), CopyWith()])
class NestedBundle with _$NestedBundle {
  /// Creates a [NestedBundle].
  const NestedBundle({required this.groups, required this.metrics});

  /// Groups.
  final List<List<String>> groups;

  /// Metrics.
  final Map<String, List<int>> metrics;
}

/// Address model for the product showcase example.
@Derive([ToString(), Eq(), CopyWith()])
class Address with _$Address {
  /// Creates an [Address].
  const Address({required this.city, required this.line1});

  /// City.
  final String city;

  /// Line1.
  final String line1;
}

/// Profile model for the product showcase example.
@Derive([ToString(), Eq(), CopyWith()])
class Profile with _$Profile {
  /// Creates a [Profile].
  const Profile({
    required this.name,
    required this.address,
    this.nickname,
    this.mailingAddress,
  });

  /// Name.
  final String name;

  /// Nickname.
  final String? nickname;

  /// Address.
  final Address address;

  /// Mailing address.
  final Address? mailingAddress;
}
