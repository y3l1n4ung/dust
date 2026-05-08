/// API User model (from JSONPlaceholder API)
class ApiUser {
  final int id;
  final String name;
  final String username;
  final String email;
  final String phone;
  final String website;
  final String company;
  final String city;

  const ApiUser({
    required this.id,
    required this.name,
    required this.username,
    required this.email,
    required this.phone,
    required this.website,
    required this.company,
    required this.city,
  });

  factory ApiUser.fromJson(Map<String, dynamic> json) {
    return ApiUser(
      id: json['id'] as int,
      name: json['name'] as String,
      username: json['username'] as String,
      email: json['email'] as String,
      phone: json['phone'] as String,
      website: json['website'] as String,
      company: (json['company'] as Map<String, dynamic>)['name'] as String,
      city: (json['address'] as Map<String, dynamic>)['city'] as String,
    );
  }
}
