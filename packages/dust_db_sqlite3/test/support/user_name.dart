import 'package:dust_db_runtime/dust_db_runtime.dart';

final class UserName {
  const UserName({required this.id, required this.name});

  final int id;
  final String name;

  static UserName fromRow(Row row) {
    return UserName(id: row.read<int>('id'), name: row.read<String>('name'));
  }
}
