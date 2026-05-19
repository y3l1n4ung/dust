import 'package:flutter/material.dart' hide Route;

import '../app_state.dart';
import '../layout/app_shell.dart';
import '../route.dart';
import 'page_scaffold.dart';

@Route(
  '/billing/invoices/:invoiceId',
  name: 'invoice',
  shell: AppShell,
  guards: [BillingGuard],
)
class InvoicePage extends StatelessWidget {
  const InvoicePage({
    required this.invoiceId,
    this.download = false,
    super.key,
  });

  final String invoiceId;
  final bool download;

  @override
  Widget build(BuildContext context) {
    return PageScaffold(
      title: 'Invoice $invoiceId',
      subtitle:
          'Nested billing route protected by a feature guard. The download flag is restored from a bool query parameter.',
      badges: const [
        StatusPill('BillingGuard', icon: Icons.policy_outlined),
        StatusPill('Nested path', icon: Icons.account_tree_outlined),
      ],
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          InfoCard(
            title: 'Invoice route params',
            icon: Icons.receipt_long_outlined,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('Download requested: $download'),
                const SizedBox(height: 12),
                CodePill('/billing/invoices/$invoiceId?download=$download'),
              ],
            ),
          ),
          const SizedBox(height: 18),
          InfoCard(
            title: 'Feature flag',
            icon: Icons.toggle_on_outlined,
            child: ListenableBuilder(
              listenable: appSession,
              builder: (context, _) => SwitchListTile(
                title: const Text('Billing feature enabled'),
                subtitle: Text(
                  appSession.billingEnabled
                      ? 'Guard passes'
                      : 'Guard redirects to /forbidden',
                ),
                value: appSession.billingEnabled,
                onChanged: (_) => appSession.toggleBilling(),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
