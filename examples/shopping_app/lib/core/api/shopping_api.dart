import 'package:dust_http_client_annotation/dust_http_client_annotation.dart';
import 'package:derive_serde_annotation/derive_serde_annotation.dart';

import '../../features/auth/models/user.dart';
import '../../features/products/models/product.dart';
import '../models/store_cart.dart';

part 'shopping_api.g.dart';

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class LoginRequest with _$LoginRequest {
  const LoginRequest({required this.username, required this.password});

  final String username;
  final String password;

  factory LoginRequest.fromJson(Map<String, Object?> json) =>
      _$LoginRequestFromJson(json);
}

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class LoginResponse with _$LoginResponse {
  const LoginResponse({required this.token});

  final String token;

  factory LoginResponse.fromJson(Map<String, Object?> json) =>
      _$LoginResponseFromJson(json);
}

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class RegisterUserResponse with _$RegisterUserResponse {
  const RegisterUserResponse({required this.id});

  final int id;

  factory RegisterUserResponse.fromJson(Map<String, Object?> json) =>
      _$RegisterUserResponseFromJson(json);
}

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class RegisterName with _$RegisterName {
  const RegisterName({required this.firstname, required this.lastname});

  final String firstname;
  final String lastname;

  factory RegisterName.fromJson(Map<String, Object?> json) =>
      _$RegisterNameFromJson(json);
}

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class RegisterGeolocation with _$RegisterGeolocation {
  const RegisterGeolocation({required this.lat, required this.long});

  final String lat;
  final String long;

  factory RegisterGeolocation.fromJson(Map<String, Object?> json) =>
      _$RegisterGeolocationFromJson(json);
}

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class RegisterAddress with _$RegisterAddress {
  const RegisterAddress({
    required this.city,
    required this.street,
    required this.number,
    required this.zipcode,
    required this.geolocation,
  });

  final String city;
  final String street;
  final int number;
  final String zipcode;
  final RegisterGeolocation geolocation;

  factory RegisterAddress.fromJson(Map<String, Object?> json) =>
      _$RegisterAddressFromJson(json);
}

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class RegisterUserRequest with _$RegisterUserRequest {
  const RegisterUserRequest({
    required this.email,
    required this.username,
    required this.password,
    required this.name,
    required this.phone,
    required this.address,
  });

  final String email;
  final String username;
  final String password;
  final RegisterName name;
  final String phone;
  final RegisterAddress address;

  factory RegisterUserRequest.fromJson(Map<String, Object?> json) =>
      _$RegisterUserRequestFromJson(json);
}

@HttpClient(
  baseUrl: 'https://fakestoreapi.com',
  headers: {'accept': 'application/json'},
  target: DustHttpTarget.flutter,
)
abstract interface class ShoppingApi {
  factory ShoppingApi(Dio dio, {String? baseUrl}) = _$ShoppingApi;

  @GET('/products')
  Future<List<Product>> getProducts();

  @GET('/products')
  Future<List<Product>> getProductsPage({
    @Query('limit') int? limit,
    @Query('sort') String? sort,
  });

  @GET('/products/{id}')
  Future<Product> getProduct(@Path() int id);

  @GET('/products/category/{category}')
  Future<List<Product>> getProductsByCategory(
    @Path() String category, {
    @Query('limit') int? limit,
    @Query('sort') String? sort,
  });

  @GET('/products/categories')
  Future<List<String>> getCategories();

  @GET('/carts')
  Future<List<StoreCart>> getCarts({
    @Query('limit') int? limit,
    @Query('sort') String? sort,
  });

  @GET('/carts/{id}')
  Future<StoreCart> getCart(@Path() int id);

  @GET('/carts/user/{userId}')
  Future<List<StoreCart>> getUserCarts(@Path() int userId);

  @POST('/auth/login')
  @Headers({'content-type': 'application/json'})
  Future<LoginResponse> login(@Body() LoginRequest payload);

  @GET('/users/{id}')
  Future<User> getUser(@Path() int id);

  @POST('/users')
  @Headers({'content-type': 'application/json'})
  Future<RegisterUserResponse> registerUser(
    @Body() RegisterUserRequest payload,
  );
}
