import 'package:flutter/material.dart' hide Route;

import '../../../route.dart';

import '../../../shared/widgets/dialogs/confirm_dialog.dart';
import '../../../shared/widgets/snackbars/app_snackbar.dart';
import '../../auth/models/auth_state.dart';
import '../../auth/view_models/auth_view_model.dart';

@Route('/profile', name: 'profile')
class ProfileScreen extends StatelessWidget {
  const ProfileScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final authState = context.watchAuthViewModel().value;

    return Scaffold(
      appBar: AppBar(title: const Text('Profile')),
      body: authState.isAuthenticated
          ? _AuthenticatedProfile(authState: authState)
          : const _GuestProfile(),
    );
  }
}

class _AuthenticatedProfile extends StatelessWidget {
  final AuthState authState;

  const _AuthenticatedProfile({required this.authState});

  @override
  Widget build(BuildContext context) {
    final user = authState.user;

    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        const CircleAvatar(radius: 50, child: Icon(Icons.person, size: 50)),
        const SizedBox(height: 16),
        Text(
          user?.name.fullName ?? 'User',
          style: Theme.of(context).textTheme.headlineSmall,
          textAlign: TextAlign.center,
        ),
        Text(
          user?.email ?? '',
          style: Theme.of(
            context,
          ).textTheme.bodyMedium?.copyWith(color: Colors.grey),
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 32),
        Card(
          child: Column(
            children: [
              ListTile(
                leading: const Icon(Icons.person_outline),
                title: const Text('Username'),
                subtitle: Text(user?.username ?? ''),
              ),
              const Divider(height: 1),
              ListTile(
                leading: const Icon(Icons.phone_outlined),
                title: const Text('Phone'),
                subtitle: Text(user?.phone ?? ''),
              ),
              const Divider(height: 1),
              ListTile(
                leading: const Icon(Icons.shopping_bag_outlined),
                title: const Text('My Orders'),
                trailing: const Icon(Icons.chevron_right),
                onTap: () => context.navigator.orders().push(),
              ),
            ],
          ),
        ),
        const SizedBox(height: 24),
        OutlinedButton.icon(
          onPressed: () async {
            final confirmed = await ConfirmDialog.show(
              context: context,
              title: 'Logout',
              message: 'Are you sure you want to logout?',
              confirmText: 'Logout',
              isDangerous: true,
            );
            if (confirmed == true && context.mounted) {
              context.readAuthViewModel().logout();
              AppSnackbar.info(context, 'You have been logged out');
              context.navigator.products().go();
            }
          },
          icon: const Icon(Icons.logout),
          label: const Text('Logout'),
          style: OutlinedButton.styleFrom(foregroundColor: Colors.red),
        ),
      ],
    );
  }
}

class _GuestProfile extends StatelessWidget {
  const _GuestProfile();

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(
              Icons.account_circle_outlined,
              size: 100,
              color: Colors.grey,
            ),
            const SizedBox(height: 24),
            Text(
              'Welcome, Guest',
              style: Theme.of(context).textTheme.headlineSmall,
            ),
            const SizedBox(height: 8),
            Text(
              'Sign in to access your profile and order history',
              style: Theme.of(
                context,
              ).textTheme.bodyMedium?.copyWith(color: Colors.grey),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 32),
            FilledButton.icon(
              onPressed: () => context.navigator.login().go(),
              icon: const Icon(Icons.login),
              label: const Text('Sign In'),
            ),
          ],
        ),
      ),
    );
  }
}
