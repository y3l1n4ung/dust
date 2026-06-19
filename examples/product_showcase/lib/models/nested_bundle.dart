import 'package:dust_dart/derive.dart';

part 'nested_bundle.g.dart';

@Derive([ToString(), Eq(), CopyWith()])
class NestedBundle with _$NestedBundle {
  const NestedBundle({required this.groups, required this.metrics});

  final List<List<String>> groups;
  final Map<String, List<int>> metrics;
}

@Derive([ToString(), Eq(), CopyWith()])
class Address with _$Address {
  const Address({required this.city, required this.line1});

  final String city;
  final String line1;
}

@Derive([ToString(), Eq(), CopyWith()])
class Profile with _$Profile {
  const Profile({
    required this.name,
    required this.address,
    this.nickname,
    this.mailingAddress,
  });

  final String name;
  final String? nickname;
  final Address address;
  final Address? mailingAddress;
}
