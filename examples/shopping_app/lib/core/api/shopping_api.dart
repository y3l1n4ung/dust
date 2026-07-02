import 'package:dust_dart/http.dart';

import '../../features/auth/models/user.dart';
import '../../features/products/models/product.dart';
import '../models/store_cart.dart';

part 'shopping_api.g.dart';

/// Login request model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class LoginRequest with _$LoginRequest {
  /// Creates a [LoginRequest].
  const LoginRequest({required this.username, required this.password});

  /// Username.
  final String username;

  /// Password.
  final String password;

  /// Creates a [LoginRequest] from JSON.
  factory LoginRequest.fromJson(Map<String, Object?> json) =>
      _$LoginRequestFromJson(json);
}

/// Login response model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class LoginResponse with _$LoginResponse {
  /// Creates a [LoginResponse].
  const LoginResponse({required this.token});

  /// Token.
  final String token;

  /// Creates a [LoginResponse] from JSON.
  factory LoginResponse.fromJson(Map<String, Object?> json) =>
      _$LoginResponseFromJson(json);
}

/// Register user response model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class RegisterUserResponse with _$RegisterUserResponse {
  /// Creates a [RegisterUserResponse].
  const RegisterUserResponse({required this.id});

  /// Unique identifier.
  final int id;

  /// Creates a [RegisterUserResponse] from JSON.
  factory RegisterUserResponse.fromJson(Map<String, Object?> json) =>
      _$RegisterUserResponseFromJson(json);
}

/// Register name model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class RegisterName with _$RegisterName {
  /// Creates a [RegisterName].
  const RegisterName({required this.firstname, required this.lastname});

  /// Firstname.
  final String firstname;

  /// Lastname.
  final String lastname;

  /// Creates a [RegisterName] from JSON.
  factory RegisterName.fromJson(Map<String, Object?> json) =>
      _$RegisterNameFromJson(json);
}

/// Register geolocation model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class RegisterGeolocation with _$RegisterGeolocation {
  /// Creates a [RegisterGeolocation].
  const RegisterGeolocation({required this.lat, required this.long});

  /// Lat.
  final String lat;

  /// Long.
  final String long;

  /// Creates a [RegisterGeolocation] from JSON.
  factory RegisterGeolocation.fromJson(Map<String, Object?> json) =>
      _$RegisterGeolocationFromJson(json);
}

/// Register address model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class RegisterAddress with _$RegisterAddress {
  /// Creates a [RegisterAddress].
  const RegisterAddress({
    required this.city,
    required this.street,
    required this.number,
    required this.zipcode,
    required this.geolocation,
  });

  /// City.
  final String city;

  /// Street.
  final String street;

  /// Number.
  final int number;

  /// Zipcode.
  final String zipcode;

  /// Geolocation.
  final RegisterGeolocation geolocation;

  /// Creates a [RegisterAddress] from JSON.
  factory RegisterAddress.fromJson(Map<String, Object?> json) =>
      _$RegisterAddressFromJson(json);
}

/// Register user request model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class RegisterUserRequest with _$RegisterUserRequest {
  /// Creates a [RegisterUserRequest].
  const RegisterUserRequest({
    required this.email,
    required this.username,
    required this.password,
    required this.name,
    required this.phone,
    required this.address,
  });

  /// Email.
  final String email;

  /// Username.
  final String username;

  /// Password.
  final String password;

  /// Name.
  final RegisterName name;

  /// Phone.
  final String phone;

  /// Address.
  final RegisterAddress address;

  /// Creates a [RegisterUserRequest] from JSON.
  factory RegisterUserRequest.fromJson(Map<String, Object?> json) =>
      _$RegisterUserRequestFromJson(json);
}

/// Shopping API contract for the shopping app example.
@HttpClient(
  baseUrl: 'https://fakestoreapi.com',
  headers: {'accept': 'application/json'},
  target: HttpTarget.flutter,
)
abstract interface class ShoppingApi {
  factory ShoppingApi(Dio dio, {String? baseUrl}) = _$ShoppingApi;

  /// Gets products.
  @GET('/products')
  Future<List<Product>> getProducts();

  /// Gets a page of products.
  @GET('/products')
  Future<List<Product>> getProductsPage({
    @Query('limit') int? limit,
    @Query('sort') String? sort,
  });

  /// Gets a product.
  @GET('/products/{id}')
  Future<Product> getProduct(@Path() int id);

  /// Gets products by category.
  @GET('/products/category/{category}')
  Future<List<Product>> getProductsByCategory(
    @Path() String category, {
    @Query('limit') int? limit,
    @Query('sort') String? sort,
  });

  /// Gets product categories.
  @GET('/products/categories')
  Future<List<String>> getCategories();

  /// Gets carts.
  @GET('/carts')
  Future<List<StoreCart>> getCarts({
    @Query('limit') int? limit,
    @Query('sort') String? sort,
  });

  /// Gets a cart.
  @GET('/carts/{id}')
  Future<StoreCart> getCart(@Path() int id);

  /// Gets user carts.
  @GET('/carts/user/{userId}')
  Future<List<StoreCart>> getUserCarts(@Path() int userId);

  /// Logs in.
  @POST('/auth/login')
  @Headers({'content-type': 'application/json'})
  Future<LoginResponse> login(@Body() LoginRequest payload);

  /// Gets user.
  @GET('/users/{id}')
  Future<User> getUser(@Path() int id);

  /// Registers user.
  @POST('/users')
  @Headers({'content-type': 'application/json'})
  Future<RegisterUserResponse> registerUser(
    @Body() RegisterUserRequest payload,
  );
}
