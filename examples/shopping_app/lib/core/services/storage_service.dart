import 'package:shared_preferences/shared_preferences.dart';

class StorageService {
  const StorageService(this._prefs);

  final SharedPreferences _prefs;

  static const authTokenKey = 'auth_token';
  static const authUserKey = 'auth_user';

  String? getString(String key) => _prefs.getString(key);
  Future<bool> setString(String key, String value) =>
      _prefs.setString(key, value);
  Future<bool> remove(String key) => _prefs.remove(key);
}
