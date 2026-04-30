import 'package:derive_annotation/derive_annotation.dart';

@Derive([ToString(), Eq(), CopyWith()])
class User {
  final String id;
  final String? name;

  const User(this.id, this.name);
}

void main() {
  const user = User('user-1', 'May');
  print(user.id);
}
