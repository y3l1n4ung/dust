import 'package:dust_flutter/i18n.dart';
import 'package:flutter/material.dart';

import '../../../route.dart';

import '../models/auth_state.dart';
import '../view_models/auth_view_model.dart';

@AppRoute('/login', name: 'login', guards: [])
class LoginScreen extends StatefulWidget {
  final String? redirectPath;

  const LoginScreen({super.key, this.redirectPath});

  @override
  State<LoginScreen> createState() => _LoginScreenState();
}

class _LoginScreenState extends State<LoginScreen> {
  final _formKey = GlobalKey<FormState>();
  final _usernameController = TextEditingController(text: 'mor_2314');
  final _passwordController = TextEditingController(text: '83r5^_');
  bool _obscurePassword = true;

  @override
  void dispose() {
    _usernameController.dispose();
    _passwordController.dispose();
    super.dispose();
  }

  void _login() {
    if (_formKey.currentState!.validate()) {
      context.readAuthViewModel().login(
            _usernameController.text,
            _passwordController.text,
          );
    }
  }

  @override
  Widget build(BuildContext context) {
    // Use watch pattern for UI updates only
    final state = context.watchAuthViewModel().value;

    return Scaffold(
      appBar: AppBar(backgroundColor: Colors.transparent, elevation: 0),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: Form(
            key: _formKey,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                const SizedBox(height: 32),
                const Icon(
                  Icons.shopping_bag,
                  size: 80,
                  color: Colors.deepPurple,
                ),
                const SizedBox(height: 24),
                TranslatedText(
                  'shop_welcome_back',
                  defaultText: 'Welcome Back',
                  style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 8),
                TranslatedText(
                  'shop_sign_in_subtitle',
                  defaultText: 'Sign in to continue shopping',
                  style: Theme.of(
                    context,
                  ).textTheme.bodyMedium?.copyWith(color: Colors.grey),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 48),
                TextFormField(
                  controller: _usernameController,
                  decoration: InputDecoration(
                    labelText: context.tr(
                      'shop_username',
                      defaultText: 'Username',
                    ),
                    prefixIcon: const Icon(Icons.person),
                    border: const OutlineInputBorder(),
                  ),
                  textInputAction: TextInputAction.next,
                  validator: (value) {
                    if (value == null || value.isEmpty) {
                      return context.tr(
                        'shop_enter_username',
                        defaultText: 'Please enter username',
                      );
                    }
                    return null;
                  },
                ),
                const SizedBox(height: 16),
                TextFormField(
                  controller: _passwordController,
                  obscureText: _obscurePassword,
                  decoration: InputDecoration(
                    labelText: context.tr(
                      'shop_password',
                      defaultText: 'Password',
                    ),
                    prefixIcon: const Icon(Icons.lock),
                    border: const OutlineInputBorder(),
                    suffixIcon: IconButton(
                      icon: Icon(
                        _obscurePassword
                            ? Icons.visibility_off
                            : Icons.visibility,
                      ),
                      onPressed: () {
                        setState(() {
                          _obscurePassword = !_obscurePassword;
                        });
                      },
                    ),
                  ),
                  textInputAction: TextInputAction.done,
                  onFieldSubmitted: (_) => _login(),
                  validator: (value) {
                    if (value == null || value.isEmpty) {
                      return context.tr(
                        'shop_enter_password',
                        defaultText: 'Please enter password',
                      );
                    }
                    return null;
                  },
                ),
                if (state.status == AuthStatus.error) ...[
                  const SizedBox(height: 16),
                  Container(
                    padding: const EdgeInsets.all(12),
                    decoration: BoxDecoration(
                      color: Colors.red.withAlpha(25),
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: Row(
                      children: [
                        const Icon(Icons.error_outline, color: Colors.red),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Text(
                            state.errorMessage ??
                                context.tr(
                                  'shop_login_failed',
                                  defaultText: 'Login failed',
                                ),
                            style: const TextStyle(color: Colors.red),
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
                const SizedBox(height: 24),
                FilledButton(
                  onPressed: state.status == AuthStatus.loading ? null : _login,
                  style: FilledButton.styleFrom(
                    padding: const EdgeInsets.symmetric(vertical: 16),
                  ),
                  child: state.status == AuthStatus.loading
                      ? const SizedBox(
                          height: 20,
                          width: 20,
                          child: CircularProgressIndicator(
                            strokeWidth: 2,
                            color: Colors.white,
                          ),
                        )
                      : const TranslatedText(
                          'shop_sign_in',
                          defaultText: 'Sign In',
                        ),
                ),
                const SizedBox(height: 16),
                Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    const TranslatedText(
                      'shop_no_account',
                      defaultText: "Don't have an account?",
                    ),
                    TextButton(
                      onPressed: () {
                        if (widget.redirectPath != null) {
                          context.navigator
                              .register(redirectPath: widget.redirectPath)
                              .go();
                        } else {
                          context.navigator.register().go();
                        }
                      },
                      child: const TranslatedText(
                        'shop_sign_up',
                        defaultText: 'Sign Up',
                      ),
                    ),
                  ],
                ),
                const Divider(height: 32),
                TextButton(
                  onPressed: () => context.navigator.products().go(),
                  child: const TranslatedText(
                    'shop_continue_as_guest',
                    defaultText: 'Continue as Guest',
                  ),
                ),
                const SizedBox(height: 24),
                Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: Colors.deepPurple.withAlpha(25),
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Row(
                    children: [
                      const Icon(Icons.info_outline, color: Colors.deepPurple),
                      const SizedBox(width: 8),
                      Expanded(
                        child: TranslatedText(
                          'shop_demo_credentials',
                          defaultText: 'Demo credentials are pre-filled',
                          style: Theme.of(context)
                              .textTheme
                              .bodySmall
                              ?.copyWith(color: Colors.deepPurple),
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
