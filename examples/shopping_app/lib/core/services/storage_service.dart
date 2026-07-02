import 'package:shared_preferences/shared_preferences.dart';

/// Storage service model for the shopping app example.
class StorageService {
  /// Creates a [StorageService].
  const StorageService(this._prefs);

  final SharedPreferences _prefs;

  /// Auth token key.
  static const authTokenKey = 'auth_token';

  /// Auth user key.
  static const authUserKey = 'auth_user';

  /// Gets string.
  String? getString(String key) => _prefs.getString(key);

  /// Sets string.
  Future<bool> setString(String key, String value) =>
      _prefs.setString(key, value);

  /// Removes.
  Future<bool> remove(String key) => _prefs.remove(key);
}
