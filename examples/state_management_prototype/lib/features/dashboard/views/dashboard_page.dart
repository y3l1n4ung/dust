import 'package:flutter/material.dart';
import 'package:state_management_prototype/features/session/view_models/session_view_model.dart';
import 'package:state_management_prototype/features/shell/view_models/shell_view_model.dart';
import 'package:state_management_prototype/features/tasks/view_models/task_board_view_model.dart';
import 'package:state_management_prototype/router/app_route_path.dart';
import 'package:state_management_prototype/router/navigation_view_model_scope.dart';
import 'package:state_management_prototype/shared/models/remote_todo.dart';

class DashboardPage extends StatelessWidget {
  const DashboardPage({super.key});

  @override
  Widget build(BuildContext context) {
    final sessionViewModel = context.watchSessionViewModel();
    final taskBoardViewModel = context.watchTaskBoardViewModel();
    final taskState = taskBoardViewModel.value;

    return SingleChildScrollView(
      padding: const EdgeInsets.fromLTRB(20, 12, 20, 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (sessionViewModel.errorMessage != null)
            _PageBanner(message: sessionViewModel.errorMessage!),
          if (taskBoardViewModel.errorMessage != null)
            Padding(
              padding: const EdgeInsets.only(top: 12),
              child: _PageBanner(message: taskBoardViewModel.errorMessage!),
            ),
          _HeroCard(
            title: 'Project Dashboard',
            subtitle: sessionViewModel.owner == null
                ? 'Loading workspace...'
                : 'Project coordination for ${sessionViewModel.owner!.company.name}',
            initials: sessionViewModel.owner?.initials ?? '…',
            lastUpdated: 'Just now',
            isRefreshing:
                sessionViewModel.isRefreshing ||
                taskBoardViewModel.isRefreshing,
            onRefresh: () => Future.wait([
              context.readSessionViewModel().refresh(),
              context.readTaskBoardViewModel().refresh(),
            ]),
          ),
          const SizedBox(height: 20),
          Wrap(
            spacing: 12,
            runSpacing: 12,
            children: [
              _MetricCard(
                label: 'Total Tasks',
                value: '${taskState.todos.length}',
              ),
              _MetricCard(
                label: 'In Progress',
                value: '${taskState.pendingCount}',
              ),
              _MetricCard(
                label: 'Success Rate',
                value: taskState.completionLabel,
              ),
            ],
          ),
          const SizedBox(height: 24),
          Text(
            'Action Required',
            style: Theme.of(
              context,
            ).textTheme.titleLarge?.copyWith(fontWeight: FontWeight.w800),
          ),
          const SizedBox(height: 12),
          if (taskState.spotlightTodos.isEmpty)
            const _EmptyCard(
              title: 'No pending work',
              body: 'Refresh the feed or switch to tasks to reopen items.',
            )
          else
            for (final todo in taskState.spotlightTodos)
              Padding(
                padding: const EdgeInsets.only(bottom: 12),
                child: _SpotlightCard(
                  todo: todo,
                  onOpen: () => context.readTaskBoardViewModel().spotlightTodo(
                    todo,
                    context.readShellViewModel(),
                  ),
                ),
              ),
          const SizedBox(height: 24),
          Text(
            'Recent posts',
            style: Theme.of(
              context,
            ).textTheme.titleLarge?.copyWith(fontWeight: FontWeight.w800),
          ),
          const SizedBox(height: 12),
          if (sessionViewModel.isPostsLoading)
            const Center(child: CircularProgressIndicator())
          else
            for (final post in sessionViewModel.posts)
              Padding(
                padding: const EdgeInsets.only(bottom: 12),
                child: Card(
                  child: ListTile(
                    title: Text(post.title),
                    subtitle: Text(
                      post.body,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                    trailing: const Icon(Icons.chevron_right),
                    onTap: () => NavigationViewModelScope.read(
                      context,
                    ).push(PostDetailPath(post.id)),
                  ),
                ),
              ),
        ],
      ),
    );
  }
}

class _HeroCard extends StatelessWidget {
  const _HeroCard({
    required this.title,
    required this.subtitle,
    required this.initials,
    required this.lastUpdated,
    required this.isRefreshing,
    required this.onRefresh,
  });

  final String title;
  final String subtitle;
  final String initials;
  final String lastUpdated;
  final bool isRefreshing;
  final Future<void> Function() onRefresh;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Row(
          children: [
            CircleAvatar(
              radius: 28,
              backgroundColor: scheme.primary,
              foregroundColor: scheme.onPrimary,
              child: Text(initials, style: const TextStyle(fontSize: 20)),
            ),
            const SizedBox(width: 16),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    title,
                    style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                      fontWeight: FontWeight.w900,
                    ),
                  ),
                  const SizedBox(height: 6),
                  Text(subtitle, style: Theme.of(context).textTheme.bodyMedium),
                  const SizedBox(height: 4),
                  Text(
                    'Updated $lastUpdated',
                    style: Theme.of(
                      context,
                    ).textTheme.labelSmall?.copyWith(color: scheme.outline),
                  ),
                ],
              ),
            ),
            FilledButton.tonalIcon(
              onPressed: isRefreshing ? null : () => onRefresh(),
              icon: isRefreshing
                  ? const SizedBox.square(
                      dimension: 16,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Icon(Icons.sync),
              label: const Text('Refresh'),
            ),
          ],
        ),
      ),
    );
  }
}

class _PageBanner extends StatelessWidget {
  const _PageBanner({required this.message});

  final String message;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.errorContainer,
        borderRadius: BorderRadius.circular(18),
      ),
      child: Text(
        message,
        style: TextStyle(color: Theme.of(context).colorScheme.onErrorContainer),
      ),
    );
  }
}

class _MetricCard extends StatelessWidget {
  const _MetricCard({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 150,
      child: Card(
        child: Padding(
          padding: const EdgeInsets.all(18),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(label, style: Theme.of(context).textTheme.labelLarge),
              const SizedBox(height: 10),
              Text(
                value,
                style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                  fontWeight: FontWeight.w900,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _SpotlightCard extends StatelessWidget {
  const _SpotlightCard({required this.todo, required this.onOpen});

  final RemoteTodo todo;
  final VoidCallback onOpen;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: ListTile(
        contentPadding: const EdgeInsets.symmetric(horizontal: 20, vertical: 8),
        leading: Icon(
          Icons.radar,
          color: Theme.of(context).colorScheme.primary,
        ),
        title: Text(todo.title, maxLines: 2, overflow: TextOverflow.ellipsis),
        subtitle: Text('${todo.lane} lane • ${todo.priority} priority'),
        trailing: FilledButton.tonal(
          onPressed: onOpen,
          child: const Text('Inspect'),
        ),
      ),
    );
  }
}

class _EmptyCard extends StatelessWidget {
  const _EmptyCard({required this.title, required this.body});

  final String title;
  final String body;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(title, style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 8),
            Text(body, style: Theme.of(context).textTheme.bodyMedium),
          ],
        ),
      ),
    );
  }
}
