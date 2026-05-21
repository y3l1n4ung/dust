import 'package:dio/dio.dart';

import '../../features/auth/models/user.dart';
import '../../features/products/models/product.dart';
import '../api/shopping_api.dart';
import '../logging/logger.dart';

abstract interface class ShoppingRepository {
  Future<List<Product>> getProducts();
  Future<Product> getProduct(int id);
  Future<List<String>> getCategories();
  Future<String> login(String username, String password);
  Future<User> getUser(int id);
  Future<int> registerUser({
    required String email,
    required String username,
    required String password,
    required String firstName,
    required String lastName,
    required String phone,
  });
}

final class LiveShoppingRepository implements ShoppingRepository {
  LiveShoppingRepository({ShoppingApi? api}) : _api = api ?? ShoppingApi(Dio());

  final ShoppingApi _api;

  @override
  Future<List<Product>> getProducts() async {
    logger.apiRequest('GET', '/products');
    final products = await _api.getProducts();
    logger.info('API', 'Loaded ${products.length} products');
    return products;
  }

  @override
  Future<Product> getProduct(int id) async {
    logger.apiRequest('GET', '/products/$id');
    return _api.getProduct(id);
  }

  @override
  Future<List<String>> getCategories() async {
    logger.apiRequest('GET', '/products/categories');
    return _api.getCategories();
  }

  @override
  Future<String> login(String username, String password) async {
    logger.apiRequest('POST', '/auth/login', {'username': username});
    final response = await _api.login(
      LoginRequest(username: username, password: password),
    );
    logger.info('API', 'Login successful for user: $username');
    return response.token;
  }

  @override
  Future<User> getUser(int id) {
    logger.apiRequest('GET', '/users/$id');
    return _api.getUser(id);
  }

  @override
  Future<int> registerUser({
    required String email,
    required String username,
    required String password,
    required String firstName,
    required String lastName,
    required String phone,
  }) async {
    logger.apiRequest('POST', '/users', {'username': username, 'email': email});
    final response = await _api.registerUser(
      RegisterUserRequest(
        email: email,
        username: username,
        password: password,
        name: RegisterName(firstname: firstName, lastname: lastName),
        phone: phone,
        address: const RegisterAddress(
          city: '',
          street: '',
          number: 0,
          zipcode: '',
          geolocation: RegisterGeolocation(lat: '0', long: '0'),
        ),
      ),
    );
    logger.info('API', 'Registration successful for user: $username');
    return response.id;
  }
}
