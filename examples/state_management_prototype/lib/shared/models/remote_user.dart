import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'remote_user.g.dart';

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class RemoteCompany with _$RemoteCompanyDust {
  const RemoteCompany({
    required this.name,
    required this.catchPhrase,
  });

  final String name;
  final String catchPhrase;

  factory RemoteCompany.fromJson(Map<String, Object?> json) =>
      _$RemoteCompanyFromJson(json);
}

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class RemoteUser with _$RemoteUserDust {
  const RemoteUser({
    required this.id,
    required this.name,
    required this.username,
    required this.email,
    required this.phone,
    required this.website,
    required this.company,
  });

  final int id;
  final String name;
  final String username;
  final String email;
  final String phone;
  final String website;
  final RemoteCompany company;

  factory RemoteUser.fromJson(Map<String, Object?> json) =>
      _$RemoteUserFromJson(json);

  String get initials {
    final parts = name.split(' ').where((part) => part.isNotEmpty).toList();
    if (parts.isEmpty) {
      return '?';
    }
    if (parts.length == 1) {
      return parts.first.substring(0, 1).toUpperCase();
    }
    return '${parts.first.substring(0, 1)}${parts.last.substring(0, 1)}'
        .toUpperCase();
  }

  String get websiteLabel => website.replaceFirst(RegExp(r'^https?://'), '');
}
