import 'package:flutter/material.dart' hide Route;

import '../../../route.dart';
import '../../../shared/widgets/dialogs/loading_dialog.dart';
import '../../../shared/widgets/snackbars/app_snackbar.dart';
import '../../cart/view_models/cart_view_model.dart';
import '../../orders/models/order.dart';
import '../../orders/view_models/orders_view_model.dart';
import '../models/checkout_shipping_request.dart';
import '../models/checkout_state.dart';
import '../view_models/checkout_view_model.dart';
import 'checkout_order_summary.dart';

@Route(
  '/checkout',
  name: 'checkout',
  transition: BottomToTopPageTransitionsBuilder(),
  fullscreenDialog: true,
)
class CheckoutScreen extends StatefulWidget {
  const CheckoutScreen({super.key});

  @override
  State<CheckoutScreen> createState() => _CheckoutScreenState();
}

class _CheckoutScreenState extends State<CheckoutScreen> {
  final _formKey = GlobalKey<FormState>();
  final _nameController = TextEditingController();
  final _addressController = TextEditingController();
  final _cityController = TextEditingController();
  final _zipController = TextEditingController();
  final _phoneController = TextEditingController();
  final _couponController = TextEditingController();

  @override
  void dispose() {
    _nameController.dispose();
    _addressController.dispose();
    _cityController.dispose();
    _zipController.dispose();
    _phoneController.dispose();
    _couponController.dispose();
    super.dispose();
  }

  Future<void> _placeOrder() async {
    if (!_formKey.currentState!.validate()) return;
    final request = _shippingRequest();

    final cartState = context.readCartViewModel().state;
    final checkoutVM = context.readCheckoutViewModel();
    final ordersVM = context.readOrdersViewModel();

    // Update shipping address
    checkoutVM.updateShippingAddress(
      ShippingAddress(
        fullName: request.fullName,
        address: request.address,
        city: request.city,
        zipCode: request.zipCode,
        phone: request.phone,
      ),
    );

    // Show loading dialog
    LoadingDialog.show(context: context, message: 'Processing your order...');

    // Process checkout
    final orderId = await checkoutVM.processCheckout(
      cartState.items,
      cartState.totalPrice,
    );

    // Hide loading dialog
    if (mounted) {
      LoadingDialog.hide(context);
    }

    if (orderId != null && mounted) {
      // Place order
      ordersVM.placeOrder(
        items: cartState.items,
        totalAmount: cartState.totalPrice,
        shippingAddress: checkoutVM.state.shippingAddress!,
      );

      // Clear cart
      context.readCartViewModel().clearCart();

      // Reset checkout
      checkoutVM.reset();

      // Navigate to confirmation
      context.navigator.orderConfirmation(orderId: orderId).go();
    } else if (mounted) {
      AppSnackbar.error(
        context,
        checkoutVM.state.errorMessage ?? 'Failed to place order',
      );
    }
  }

  CheckoutShippingRequest _shippingRequest() {
    return CheckoutShippingRequest(
      fullName: _nameController.text,
      address: _addressController.text,
      city: _cityController.text,
      zipCode: _zipController.text,
      phone: _phoneController.text,
    );
  }

  Future<void> _applyCoupon() async {
    final subtotal = context.readCartViewModel().state.totalPrice;
    await context.readCheckoutViewModel().applyCoupon(
      subtotal: subtotal,
      couponCode: _couponController.text,
    );
    if (!mounted) return;

    final quote = context.readCheckoutViewModel().state.quote;
    if (quote?.message != null) {
      AppSnackbar.info(context, quote!.message!);
    }
  }

  @override
  Widget build(BuildContext context) {
    final cartState = context.watchCartViewModel().value;
    final checkoutState = context.watchCheckoutViewModel().value;

    return Scaffold(
      appBar: AppBar(title: const Text('Checkout')),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Form(
          key: _formKey,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'Shipping Address',
                style: Theme.of(context).textTheme.titleLarge,
              ),
              const SizedBox(height: 16),
              TextFormField(
                controller: _nameController,
                decoration: const InputDecoration(
                  labelText: 'Full Name',
                  border: OutlineInputBorder(),
                ),
                validator: validateCheckoutShippingRequestFullNameInput,
              ),
              const SizedBox(height: 12),
              TextFormField(
                controller: _addressController,
                decoration: const InputDecoration(
                  labelText: 'Address',
                  border: OutlineInputBorder(),
                ),
                validator: validateCheckoutShippingRequestAddressInput,
              ),
              const SizedBox(height: 12),
              Row(
                children: [
                  Expanded(
                    flex: 2,
                    child: TextFormField(
                      controller: _cityController,
                      decoration: const InputDecoration(
                        labelText: 'City',
                        border: OutlineInputBorder(),
                      ),
                      validator: validateCheckoutShippingRequestCityInput,
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: TextFormField(
                      controller: _zipController,
                      decoration: const InputDecoration(
                        labelText: 'ZIP',
                        border: OutlineInputBorder(),
                      ),
                      validator: validateCheckoutShippingRequestZipCodeInput,
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 12),
              TextFormField(
                controller: _phoneController,
                decoration: const InputDecoration(
                  labelText: 'Phone',
                  border: OutlineInputBorder(),
                ),
                keyboardType: TextInputType.phone,
                validator: validateCheckoutShippingRequestPhoneInput,
              ),
              const SizedBox(height: 24),
              Text(
                'Order Summary',
                style: Theme.of(context).textTheme.titleLarge,
              ),
              const SizedBox(height: 16),
              CheckoutOrderSummary(
                cartState: cartState,
                checkoutState: checkoutState,
                couponController: _couponController,
                onApplyCoupon: _applyCoupon,
              ),
              if (checkoutState.status == CheckoutStatus.error) ...[
                const SizedBox(height: 16),
                Text(
                  checkoutState.errorMessage ?? 'An error occurred',
                  style: const TextStyle(color: Colors.red),
                ),
              ],
              const SizedBox(height: 24),
              SizedBox(
                width: double.infinity,
                child: FilledButton(
                  onPressed: checkoutState.status == CheckoutStatus.processing
                      ? null
                      : _placeOrder,
                  child: checkoutState.status == CheckoutStatus.processing
                      ? const SizedBox(
                          height: 20,
                          width: 20,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Text('Place Order'),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
