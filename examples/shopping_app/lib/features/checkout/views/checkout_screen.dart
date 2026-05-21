import 'package:flutter/material.dart' hide Route;

import '../../../route.dart';

import '../../../shared/widgets/dialogs/loading_dialog.dart';
import '../../../shared/widgets/snackbars/app_snackbar.dart';
import '../../cart/view_models/cart_view_model.dart';
import '../../orders/models/order.dart';
import '../../orders/view_models/orders_view_model.dart';
import '../models/checkout_state.dart';
import '../view_models/checkout_view_model.dart';

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

  @override
  void dispose() {
    _nameController.dispose();
    _addressController.dispose();
    _cityController.dispose();
    _zipController.dispose();
    _phoneController.dispose();
    super.dispose();
  }

  Future<void> _placeOrder() async {
    if (!_formKey.currentState!.validate()) return;

    final cartState = context.readCartViewModel().state;
    final checkoutVM = context.readCheckoutViewModel();
    final ordersVM = context.readOrdersViewModel();

    // Update shipping address
    checkoutVM.updateShippingAddress(
      ShippingAddress(
        fullName: _nameController.text,
        address: _addressController.text,
        city: _cityController.text,
        zipCode: _zipController.text,
        phone: _phoneController.text,
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
      context.routes.orderConfirmation(orderId: orderId).go();
    } else if (mounted) {
      AppSnackbar.error(
        context,
        checkoutVM.state.errorMessage ?? 'Failed to place order',
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final cartState = context.cartViewModel.state;
    final checkoutState = context.checkoutViewModel.state;

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
                validator: (v) => v?.isEmpty ?? true ? 'Required' : null,
              ),
              const SizedBox(height: 12),
              TextFormField(
                controller: _addressController,
                decoration: const InputDecoration(
                  labelText: 'Address',
                  border: OutlineInputBorder(),
                ),
                validator: (v) => v?.isEmpty ?? true ? 'Required' : null,
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
                      validator: (v) => v?.isEmpty ?? true ? 'Required' : null,
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
                      validator: (v) => v?.isEmpty ?? true ? 'Required' : null,
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
                validator: (v) => v?.isEmpty ?? true ? 'Required' : null,
              ),
              const SizedBox(height: 24),
              Text(
                'Order Summary',
                style: Theme.of(context).textTheme.titleLarge,
              ),
              const SizedBox(height: 16),
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    children: [
                      ...cartState.items.map(
                        (item) => Padding(
                          padding: const EdgeInsets.only(bottom: 8),
                          child: Row(
                            mainAxisAlignment: MainAxisAlignment.spaceBetween,
                            children: [
                              Expanded(
                                child: Text(
                                  '${item.product.title} x${item.quantity}',
                                  maxLines: 1,
                                  overflow: TextOverflow.ellipsis,
                                ),
                              ),
                              Text('\$${item.totalPrice.toStringAsFixed(2)}'),
                            ],
                          ),
                        ),
                      ),
                      const Divider(),
                      Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          Text(
                            'Total',
                            style: Theme.of(context).textTheme.titleMedium,
                          ),
                          Text(
                            '\$${cartState.totalPrice.toStringAsFixed(2)}',
                            style: Theme.of(context).textTheme.titleLarge
                                ?.copyWith(
                                  fontWeight: FontWeight.bold,
                                  color: Colors.deepPurple,
                                ),
                          ),
                        ],
                      ),
                    ],
                  ),
                ),
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
