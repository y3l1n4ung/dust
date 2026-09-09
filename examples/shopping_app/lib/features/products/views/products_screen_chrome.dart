part of 'products_screen.dart';

// The app bar and drawer chrome around the product list.

class _LanguageToggleButton extends StatelessWidget {
  const _LanguageToggleButton();

  @override
  Widget build(BuildContext context) {
    final i18n = I18nScope.of(context);
    final nextLocale = i18n.locale == 'en' ? 'my' : 'en';
    return TextButton.icon(
      onPressed: () => i18n.setLocale(nextLocale),
      icon: const Icon(Icons.language),
      label: Text(i18n.locale.toUpperCase()),
    );
  }
}

class _CartIconButton extends StatefulWidget {
  final int itemCount;

  const _CartIconButton({required this.itemCount});

  @override
  State<_CartIconButton> createState() => _CartIconButtonState();
}

class _CartIconButtonState extends State<_CartIconButton>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _scaleAnimation;
  int _previousCount = 0;

  @override
  void initState() {
    super.initState();
    _previousCount = widget.itemCount;
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 300),
    );
    _scaleAnimation = TweenSequence<double>([
      TweenSequenceItem(tween: Tween(begin: 1.0, end: 1.3), weight: 1),
      TweenSequenceItem(tween: Tween(begin: 1.3, end: 1.0), weight: 1),
    ]).animate(CurvedAnimation(parent: _controller, curve: Curves.easeInOut));
  }

  @override
  void didUpdateWidget(_CartIconButton oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.itemCount > _previousCount) {
      _controller.forward(from: 0);
    }
    _previousCount = widget.itemCount;
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return ScaleTransition(
      scale: _scaleAnimation,
      child: Stack(
        children: [
          IconButton(
            icon: const Icon(Icons.shopping_cart),
            onPressed: () => context.navigator.cart().push(),
          ),
          if (widget.itemCount > 0)
            Positioned(
              right: 4,
              top: 4,
              child: Container(
                padding: const EdgeInsets.all(4),
                decoration: const BoxDecoration(
                  color: Colors.red,
                  shape: BoxShape.circle,
                ),
                constraints: const BoxConstraints(minWidth: 18, minHeight: 18),
                child: Text(
                  widget.itemCount > 99 ? '99+' : '${widget.itemCount}',
                  style: const TextStyle(
                    color: Colors.white,
                    fontSize: 10,
                    fontWeight: FontWeight.bold,
                  ),
                  textAlign: TextAlign.center,
                ),
              ),
            ),
        ],
      ),
    );
  }
}

class _AppDrawer extends StatelessWidget {
  const _AppDrawer();

  @override
  Widget build(BuildContext context) {
    final authState = context.watchAuthViewModel().value;

    return Drawer(
      child: ListView(
        padding: EdgeInsets.zero,
        children: [
          DrawerHeader(
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.primary,
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                const CircleAvatar(
                  radius: 30,
                  child: Icon(Icons.person, size: 30),
                ),
                const SizedBox(height: 12),
                if (authState.isAuthenticated)
                  Text(
                    authState.user?.name.fullName ??
                        context.tr('shop_user', defaultText: 'User'),
                    style: const TextStyle(
                      color: Colors.white,
                      fontSize: 18,
                      fontWeight: FontWeight.bold,
                    ),
                  )
                else
                  const TranslatedText(
                    'shop_guest',
                    defaultText: 'Guest',
                    style: TextStyle(
                      color: Colors.white,
                      fontSize: 18,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                if (authState.isAuthenticated)
                  Text(
                    authState.user?.email ?? '',
                    style: TextStyle(
                      color: Colors.white.withAlpha(200),
                      fontSize: 14,
                    ),
                  ),
              ],
            ),
          ),
          ListTile(
            leading: const Icon(Icons.home),
            title: const TranslatedText('shop_title', defaultText: 'Shop'),
            onTap: () {
              Navigator.pop(context);
              context.navigator.products().go();
            },
          ),
          ListTile(
            leading: const Icon(Icons.shopping_cart),
            title: const TranslatedText('shop_cart', defaultText: 'Cart'),
            onTap: () {
              Navigator.pop(context);
              context.navigator.cart().push();
            },
          ),
          ListTile(
            leading: const Icon(Icons.favorite),
            title:
                const TranslatedText('shop_wishlist', defaultText: 'Wishlist'),
            onTap: () {
              Navigator.pop(context);
              context.navigator.wishlist().push();
            },
          ),
          ListTile(
            leading: const Icon(Icons.cloud_queue),
            title: const TranslatedText(
              'shop_remote_carts',
              defaultText: 'FakeStore Carts',
            ),
            onTap: () {
              Navigator.pop(context);
              context.navigator.demoCarts().push();
            },
          ),
          ListTile(
            leading: const Icon(Icons.support_agent),
            title: const TranslatedText(
              'shop_support_chat',
              defaultText: 'Support Chat',
            ),
            onTap: () {
              Navigator.pop(context);
              context.navigator.supportChat().push();
            },
          ),
          ListTile(
            leading: const Icon(Icons.receipt_long),
            title: const TranslatedText('shop_orders', defaultText: 'Orders'),
            onTap: () {
              Navigator.pop(context);
              context.navigator.orders().push();
            },
          ),
          ListTile(
            leading: const Icon(Icons.person),
            title: const TranslatedText('shop_profile', defaultText: 'Profile'),
            onTap: () {
              Navigator.pop(context);
              context.navigator.profile().push();
            },
          ),
          const Divider(),
          if (authState.isAuthenticated)
            ListTile(
              leading: const Icon(Icons.logout, color: Colors.red),
              title: const TranslatedText(
                'shop_logout',
                defaultText: 'Logout',
                style: TextStyle(color: Colors.red),
              ),
              onTap: () async {
                final confirmed = await ConfirmDialog.show(
                  context: context,
                  title: context.tr('shop_logout', defaultText: 'Logout'),
                  message: context.tr(
                    'shop_logout_message',
                    defaultText: 'Are you sure you want to logout?',
                  ),
                  confirmText: context.tr('shop_logout', defaultText: 'Logout'),
                  isDangerous: true,
                );
                if (confirmed == true && context.mounted) {
                  context.readAuthViewModel().logout();
                  Navigator.pop(context);
                }
              },
            )
          else
            ListTile(
              leading: const Icon(Icons.login),
              title: const TranslatedText(
                'shop_sign_in',
                defaultText: 'Sign In',
              ),
              onTap: () {
                Navigator.pop(context);
                context.navigator.login().go();
              },
            ),
        ],
      ),
    );
  }
}
