import 'package:flutter/material.dart';
import 'package:state_management_prototype/features/session/models/session_state.dart';
import 'package:state_management_prototype/features/session/view_models/session_view_model.dart';
import 'package:state_management_prototype/features/tasks/models/task_board_state.dart';
import 'package:state_management_prototype/features/tasks/view_models/task_board_view_model.dart';
import 'package:state_management_prototype/features/theme/view_models/theme_view_model.dart';

class ProfilePage extends StatelessWidget {
  const ProfilePage({super.key});

  @override
  Widget build(BuildContext context) {
    final sessionViewModel = context.sessionViewModel;
    final taskBoardViewModel = context.taskBoardViewModel;
    final themeViewModel = context.themeViewModel;

    return ValueListenableBuilder<SessionState>(
      valueListenable: sessionViewModel,
      builder: (context, sessionState, child) {
        return ValueListenableBuilder<TaskBoardState>(
          valueListenable: taskBoardViewModel,
          builder: (context, taskState, child) {
            final user = sessionState.owner;

            return ListView(
              padding: const EdgeInsets.fromLTRB(20, 12, 20, 24),
              children: [
                Text(
                  'Owner profile',
                  style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                        fontWeight: FontWeight.w900,
                      ),
                ),
                const SizedBox(height: 18),
                if (sessionState.errorMessage != null)
                  Container(
                    width: double.infinity,
                    margin: const EdgeInsets.only(bottom: 18),
                    padding: const EdgeInsets.symmetric(
                      horizontal: 16,
                      vertical: 14,
                    ),
                    decoration: BoxDecoration(
                      color: Theme.of(context).colorScheme.errorContainer,
                      borderRadius: BorderRadius.circular(18),
                    ),
                    child: Text(sessionState.errorMessage!),
                  ),
                Card(
                  child: Padding(
                    padding: const EdgeInsets.all(24),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        if (user != null) ...[
                          ListTile(
                            contentPadding: EdgeInsets.zero,
                            leading: CircleAvatar(
                              radius: 24,
                              child: Text(user.initials),
                            ),
                            title: Text(
                              user.name,
                              style: Theme.of(context).textTheme.titleLarge,
                            ),
                            subtitle: Text('@${user.username}'),
                          ),
                          const Divider(height: 24),
                          _DetailRow(label: 'Email', value: user.email),
                          _DetailRow(label: 'Phone', value: user.phone),
                          _DetailRow(
                            label: 'Website',
                            value: user.websiteLabel,
                          ),
                          _DetailRow(
                            label: 'Company',
                            value: user.company.name,
                          ),
                          _DetailRow(
                            label: 'Motto',
                            value: user.company.catchPhrase,
                          ),
                        ] else
                          const Text('No profile loaded yet.'),
                      ],
                    ),
                  ),
                ),
                const SizedBox(height: 18),
                Card(
                  child: Column(
                    children: [
                      SwitchListTile(
                        value: themeViewModel.value == ThemeMode.dark,
                        title: const Text('Dark mode'),
                        subtitle: const Text(
                          'Toggle the shell theme from its own view model.',
                        ),
                        onChanged: (_) => themeViewModel.toggle(),
                      ),
                      ListTile(
                        title: const Text('Dust footprint'),
                        subtitle: const Text(
                          'Models use derive + serde. Network uses Dust '
                          'HttpClient. View models stay Flutter-native.',
                        ),
                        trailing: const Icon(Icons.auto_awesome),
                      ),
                    ],
                  ),
                ),
                const SizedBox(height: 18),
                FilledButton.tonalIcon(
                  onPressed: taskState.completedCount == 0
                      ? null
                      : taskBoardViewModel.clearCompleted,
                  icon: const Icon(Icons.cleaning_services_outlined),
                  label: const Text('Clear completed tasks'),
                ),
              ],
            );
          },
        );
      },
    );
  }
}

class _DetailRow extends StatelessWidget {
  const _DetailRow({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 88,
            child: Text(
              label,
              style: Theme.of(context).textTheme.labelLarge,
            ),
          ),
          Expanded(child: Text(value)),
        ],
      ),
    );
  }
}
