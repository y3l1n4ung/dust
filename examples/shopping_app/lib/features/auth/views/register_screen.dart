import 'package:dust_flutter/i18n.dart';
import 'package:flutter/material.dart';

import '../../../route.dart';
import '../models/auth_state.dart';
import '../models/register_request.dart';
import '../view_models/auth_view_model.dart';
import 'register_actions.dart';
import 'register_auth_error.dart';
import 'register_header.dart';

@AppRoute('/register', name: 'register', guards: [])
class RegisterScreen extends StatefulWidget {
  final String? redirectPath;

  const RegisterScreen({super.key, this.redirectPath});

  @override
  State<RegisterScreen> createState() => _RegisterScreenState();
}

class _RegisterScreenState extends State<RegisterScreen> {
  final _formKey = GlobalKey<FormState>();
  final _emailController = TextEditingController();
  final _usernameController = TextEditingController();
  final _passwordController = TextEditingController();
  final _confirmPasswordController = TextEditingController();
  final _firstNameController = TextEditingController();
  final _lastNameController = TextEditingController();
  final _phoneController = TextEditingController();
  bool _obscurePassword = true;
  bool _obscureConfirmPassword = true;

  @override
  void dispose() {
    _emailController.dispose();
    _usernameController.dispose();
    _passwordController.dispose();
    _confirmPasswordController.dispose();
    _firstNameController.dispose();
    _lastNameController.dispose();
    _phoneController.dispose();
    super.dispose();
  }

  void _register() {
    if (!_formKey.currentState!.validate()) return;
    final request = _request();

    context.readAuthViewModel().register(
          email: request.email,
          username: request.username,
          password: request.password,
          firstName: request.firstName,
          lastName: request.lastName,
          phone: request.phone,
        );
  }

  RegisterRequest _request() {
    return RegisterRequest(
      email: _emailController.text,
      username: _usernameController.text,
      password: _passwordController.text,
      confirmPassword: _confirmPasswordController.text,
      firstName: _firstNameController.text,
      lastName: _lastNameController.text,
      phone: _phoneController.text,
    );
  }

  @override
  Widget build(BuildContext context) {
    final state = context.watchAuthViewModel().value;

    return Scaffold(
      appBar: AppBar(
        backgroundColor: Colors.transparent,
        elevation: 0,
        title: const TranslatedText(
          'shop_create_account',
          defaultText: 'Create Account',
        ),
      ),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: Form(
            key: _formKey,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                const RegisterHeader(),
                const SizedBox(height: 32),

                // Name fields
                Row(
                  children: [
                    Expanded(
                      child: TextFormField(
                        controller: _firstNameController,
                        decoration: InputDecoration(
                          labelText: context.tr(
                            'shop_first_name',
                            defaultText: 'First Name',
                          ),
                          border: const OutlineInputBorder(),
                        ),
                        textInputAction: TextInputAction.next,
                        textCapitalization: TextCapitalization.words,
                        validator: validateRegisterRequestFirstNameInput,
                      ),
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: TextFormField(
                        controller: _lastNameController,
                        decoration: InputDecoration(
                          labelText: context.tr(
                            'shop_last_name',
                            defaultText: 'Last Name',
                          ),
                          border: const OutlineInputBorder(),
                        ),
                        textInputAction: TextInputAction.next,
                        textCapitalization: TextCapitalization.words,
                        validator: validateRegisterRequestLastNameInput,
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 16),

                // Email
                TextFormField(
                  controller: _emailController,
                  decoration: InputDecoration(
                    labelText: context.tr('shop_email', defaultText: 'Email'),
                    prefixIcon: const Icon(Icons.email),
                    border: const OutlineInputBorder(),
                  ),
                  keyboardType: TextInputType.emailAddress,
                  textInputAction: TextInputAction.next,
                  validator: validateRegisterRequestEmailInput,
                ),
                const SizedBox(height: 16),

                // Username
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
                  validator: validateRegisterRequestUsernameInput,
                ),
                const SizedBox(height: 16),

                // Phone
                TextFormField(
                  controller: _phoneController,
                  decoration: InputDecoration(
                    labelText: context.tr('shop_phone', defaultText: 'Phone'),
                    prefixIcon: const Icon(Icons.phone),
                    border: const OutlineInputBorder(),
                  ),
                  keyboardType: TextInputType.phone,
                  textInputAction: TextInputAction.next,
                  validator: validateRegisterRequestPhoneInput,
                ),
                const SizedBox(height: 16),

                // Password
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
                  textInputAction: TextInputAction.next,
                  validator: validateRegisterRequestPasswordInput,
                ),
                const SizedBox(height: 16),

                // Confirm Password
                TextFormField(
                  controller: _confirmPasswordController,
                  obscureText: _obscureConfirmPassword,
                  decoration: InputDecoration(
                    labelText: context.tr(
                      'shop_confirm_password',
                      defaultText: 'Confirm Password',
                    ),
                    prefixIcon: const Icon(Icons.lock_outline),
                    border: const OutlineInputBorder(),
                    suffixIcon: IconButton(
                      icon: Icon(
                        _obscureConfirmPassword
                            ? Icons.visibility_off
                            : Icons.visibility,
                      ),
                      onPressed: () {
                        setState(() {
                          _obscureConfirmPassword = !_obscureConfirmPassword;
                        });
                      },
                    ),
                  ),
                  textInputAction: TextInputAction.done,
                  onFieldSubmitted: (_) => _register(),
                  validator: (value) {
                    return validateRegisterRequestConfirmPasswordInput(
                      _request(),
                      value,
                    );
                  },
                ),

                if (state.status == AuthStatus.error)
                  const SizedBox(height: 16),
                RegisterAuthError(state: state),
                const SizedBox(height: 24),
                RegisterActions(
                  status: state.status,
                  redirectPath: widget.redirectPath,
                  onRegister: _register,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
