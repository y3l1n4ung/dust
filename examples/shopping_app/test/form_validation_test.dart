import 'package:flutter/material.dart' hide Route;
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:shopping_app/core/services/storage_service.dart';
import 'package:shopping_app/features/auth/view_models/auth_view_model.dart';
import 'package:shopping_app/features/auth/views/register_screen.dart';
import 'package:shopping_app/features/cart/view_models/cart_view_model.dart';
import 'package:shopping_app/features/checkout/view_models/checkout_view_model.dart';
import 'package:shopping_app/features/checkout/views/checkout_screen.dart';
import 'package:shopping_app/features/orders/view_models/orders_view_model.dart';

import 'support/fake_shopping_repository.dart';

void main() {
  testWidgets('register form renders generated validation errors', (
    tester,
  ) async {
    final repository = FakeShoppingRepository();
    await tester.pumpWidget(await _registerHarness(repository));
    await tester.pump();

    await _tapCreateAccount(tester);

    expect(find.text('Please enter email'), findsOneWidget);
    expect(find.text('Please enter username'), findsOneWidget);
    expect(find.text('Please enter password'), findsOneWidget);
    expect(find.text('Please confirm password'), findsOneWidget);
    expect(find.text('Please enter phone number'), findsOneWidget);
    expect(find.text('Required'), findsNWidgets(2));
    expect(repository.registerCalls, 0);
  });

  testWidgets('register form submits valid generated request', (tester) async {
    final repository = FakeShoppingRepository();
    await tester.pumpWidget(await _registerHarness(repository));
    await tester.pump();

    await tester.enterText(
      find.widgetWithText(TextFormField, 'First Name'),
      'Dust',
    );
    await tester.enterText(
      find.widgetWithText(TextFormField, 'Last Name'),
      'Dev',
    );
    await tester.enterText(
      find.widgetWithText(TextFormField, 'Email'),
      'dust@example.com',
    );
    await tester.enterText(
      find.widgetWithText(TextFormField, 'Username'),
      'dustdev',
    );
    await tester.enterText(
      find.widgetWithText(TextFormField, 'Phone'),
      '555-0100',
    );
    await tester.enterText(
      find.widgetWithText(TextFormField, 'Password'),
      'secret1',
    );
    await tester.enterText(
      find.widgetWithText(TextFormField, 'Confirm Password'),
      'secret1',
    );

    await _tapCreateAccount(tester);
    await tester.pumpAndSettle();

    expect(repository.registerCalls, 1);
    expect(repository.lastRegisteredEmail, 'dust@example.com');
  });

  testWidgets('checkout form renders generated shipping validation errors', (
    tester,
  ) async {
    final repository = FakeShoppingRepository();
    final cart = CartViewModel(const CartViewModelArgs())
      ..addToCart(FakeShoppingRepository.products.first);

    await tester.pumpWidget(
      MaterialApp(
        home: CartViewModelScope.value(
          value: cart,
          child: CheckoutViewModelScope.value(
            value: CheckoutViewModel(
              CheckoutViewModelArgs(repository: repository),
            ),
            child: OrdersViewModelScope.value(
              value: OrdersViewModel(const OrdersViewModelArgs()),
              child: const CheckoutScreen(),
            ),
          ),
        ),
      ),
    );
    await tester.pump();

    await tester.ensureVisible(find.text('Place Order'));
    await tester.tap(find.text('Place Order'));
    await tester.pump();

    expect(find.text('Required'), findsNWidgets(5));
    expect(find.text('Processing your order...'), findsNothing);
  });
}

Future<Widget> _registerHarness(FakeShoppingRepository repository) async {
  SharedPreferences.setMockInitialValues({});
  final prefs = await SharedPreferences.getInstance();
  return MaterialApp(
    home: AuthViewModelScope(
      args: (context) => AuthViewModelArgs(
        repository: repository,
        storage: StorageService(prefs),
      ),
      create: (context, args) => AuthViewModel(args),
      child: const RegisterScreen(),
    ),
  );
}

Future<void> _tapCreateAccount(WidgetTester tester) async {
  final button = find.widgetWithText(FilledButton, 'Create Account');
  await tester.ensureVisible(button);
  await tester.tap(button);
  await tester.pump();
}
