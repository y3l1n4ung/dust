import 'package:flutter/foundation.dart';

final AppSession appSession = AppSession();

final class AppSession extends ChangeNotifier {
  bool _isLoggedIn = false;
  bool _isAdmin = false;
  bool _billingEnabled = false;

  bool get isLoggedIn => _isLoggedIn;
  bool get isAdmin => _isAdmin;
  bool get billingEnabled => _billingEnabled;

  void logIn({bool admin = false}) {
    _isLoggedIn = true;
    _isAdmin = admin;
    _billingEnabled = true;
    notifyListeners();
  }

  void logOut() {
    _isLoggedIn = false;
    _isAdmin = false;
    _billingEnabled = false;
    notifyListeners();
  }

  void toggleAdmin() {
    _isAdmin = !_isAdmin;
    notifyListeners();
  }

  void toggleBilling() {
    _billingEnabled = !_billingEnabled;
    notifyListeners();
  }
}
