import 'package:flutter/material.dart';
import 'package:state_management_prototype/features/tasks/models/task_board_state.dart';
import 'package:state_management_prototype/features/tasks/view_models/task_board_view_model.dart';
import 'package:state_management_prototype/shared/models/remote_todo.dart';

class TasksPage extends StatefulWidget {
  const TasksPage({super.key});

  @override
  State<TasksPage> createState() => _TasksPageState();
}

class _TasksPageState extends State<TasksPage> {
  late final TextEditingController _queryController;

  @override
  void initState() {
    super.initState();
    _queryController = TextEditingController();
  }

  @override
  void dispose() {
    _queryController.dispose();
    super.dispose();
  }

  void _showAddTaskSheet(BuildContext context) {
    final viewModel = context.readTaskBoardViewModel();
    final titleController = TextEditingController();
    String selectedPriority = 'Medium';
    String selectedLane = 'Backlog';

    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      builder: (context) => Padding(
        padding: EdgeInsets.fromLTRB(
          24,
          24,
          24,
          MediaQuery.of(context).viewInsets.bottom + 24,
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'New Task',
              style: Theme.of(
                context,
              ).textTheme.headlineSmall?.copyWith(fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 18),
            TextField(
              controller: titleController,
              autofocus: true,
              decoration: const InputDecoration(
                labelText: 'Task Title',
                hintText: 'What needs to be done?',
              ),
            ),
            const SizedBox(height: 18),
            Row(
              children: [
                Expanded(
                  child: DropdownButtonFormField<String>(
                    initialValue: selectedLane,
                    decoration: const InputDecoration(labelText: 'Lane'),
                    items: ['Backlog', 'In Progress', 'Testing', 'Done']
                        .map((l) => DropdownMenuItem(value: l, child: Text(l)))
                        .toList(),
                    onChanged: (v) => selectedLane = v!,
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: DropdownButtonFormField<String>(
                    initialValue: selectedPriority,
                    decoration: const InputDecoration(labelText: 'Priority'),
                    items: ['Low', 'Medium', 'High']
                        .map((p) => DropdownMenuItem(value: p, child: Text(p)))
                        .toList(),
                    onChanged: (v) => selectedPriority = v!,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 24),
            SizedBox(
              width: double.infinity,
              child: FilledButton(
                onPressed: () {
                  if (titleController.text.isNotEmpty) {
                    viewModel.addTodo(
                      titleController.text,
                      selectedLane,
                      selectedPriority,
                    );
                    Navigator.pop(context);
                  }
                },
                child: const Text('Add Task'),
              ),
            ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final viewModel = context.watchTaskBoardViewModel();

    if (_queryController.text != viewModel.query) {
      _queryController.value = TextEditingValue(
        text: viewModel.query,
        selection: TextSelection.collapsed(offset: viewModel.query.length),
      );
    }

    return Scaffold(
      backgroundColor: Colors.transparent,
      floatingActionButton: FloatingActionButton(
        onPressed: () => _showAddTaskSheet(context),
        child: const Icon(Icons.add),
      ),
      body: Padding(
        padding: const EdgeInsets.fromLTRB(20, 12, 20, 0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  'Project Board',
                  style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                    fontWeight: FontWeight.w900,
                  ),
                ),
                if (viewModel.completedCount > 0)
                  TextButton.icon(
                    onPressed: () =>
                        context.readTaskBoardViewModel().clearCompleted(),
                    icon: const Icon(Icons.delete_sweep_outlined),
                    label: const Text('Clear Done'),
                  ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              'Manage and track team deliverables across different lanes.',
              style: Theme.of(context).textTheme.bodyMedium,
            ),
            const SizedBox(height: 18),
            _MetricsHeader(
              completedCount: viewModel.completedCount,
              totalCount: viewModel.todos.length,
              completionLabel: viewModel.completionLabel,
            ),
            const SizedBox(height: 18),
            if (viewModel.errorMessage != null)
              Container(
                width: double.infinity,
                margin: const EdgeInsets.only(bottom: 14),
                padding: const EdgeInsets.symmetric(
                  horizontal: 16,
                  vertical: 14,
                ),
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.errorContainer,
                  borderRadius: BorderRadius.circular(18),
                ),
                child: Text(
                  viewModel.errorMessage!,
                  style: TextStyle(
                    color: Theme.of(context).colorScheme.onErrorContainer,
                  ),
                ),
              ),
            TextField(
              controller: _queryController,
              onChanged: (val) =>
                  context.readTaskBoardViewModel().setQuery(val),
              decoration: const InputDecoration(
                prefixIcon: Icon(Icons.search),
                hintText: 'Search title or lane',
              ),
            ),
            const SizedBox(height: 14),
            Wrap(
              spacing: 10,
              children: TodoFilter.values
                  .map(
                    (filter) => ChoiceChip(
                      label: Text(_filterLabel(filter)),
                      selected: viewModel.filter == filter,
                      onSelected: (_) =>
                          context.readTaskBoardViewModel().setFilter(filter),
                    ),
                  )
                  .toList(),
            ),
            const SizedBox(height: 14),
            Expanded(
              child: RefreshIndicator(
                onRefresh: () => context.readTaskBoardViewModel().refresh(),
                child: viewModel.visibleTodos.isEmpty
                    ? ListView(
                        children: const [
                          SizedBox(height: 120),
                          _EmptyTaskState(),
                        ],
                      )
                    : ListView.separated(
                        itemCount: viewModel.visibleTodos.length,
                        separatorBuilder: (_, _) => const SizedBox(height: 12),
                        itemBuilder: (context, index) {
                          final todo = viewModel.visibleTodos[index];
                          return _TodoTile(
                            key: ValueKey(todo.id),
                            todo: todo,
                            onToggle: () => context
                                .readTaskBoardViewModel()
                                .toggleTodo(todo.id),
                            onDelete: () => context
                                .readTaskBoardViewModel()
                                .deleteTodo(todo.id),
                          );
                        },
                      ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

String _filterLabel(TodoFilter filter) => switch (filter) {
  TodoFilter.all => 'All',
  TodoFilter.open => 'Open',
  TodoFilter.done => 'Done',
};

class _MetricsHeader extends StatelessWidget {
  const _MetricsHeader({
    required this.completedCount,
    required this.totalCount,
    required this.completionLabel,
  });

  final int completedCount;
  final int totalCount;
  final String completionLabel;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final progress = totalCount == 0 ? 0.0 : completedCount / totalCount;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      '$completedCount of $totalCount completed',
                      style: theme.textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      'Overall progress tracking',
                      style: theme.textTheme.bodySmall,
                    ),
                  ],
                ),
                Text(
                  completionLabel,
                  style: theme.textTheme.headlineSmall?.copyWith(
                    fontWeight: FontWeight.w900,
                    color: theme.colorScheme.primary,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            LinearProgressIndicator(
              value: progress,
              borderRadius: BorderRadius.circular(8),
              minHeight: 8,
            ),
          ],
        ),
      ),
    );
  }
}

class _TodoTile extends StatelessWidget {
  const _TodoTile({
    super.key,
    required this.todo,
    required this.onToggle,
    required this.onDelete,
  });

  final RemoteTodo todo;
  final VoidCallback onToggle;
  final VoidCallback onDelete;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;

    return Dismissible(
      key: ValueKey('dismiss-${todo.id}'),
      direction: DismissDirection.endToStart,
      background: Container(
        alignment: Alignment.centerRight,
        padding: const EdgeInsets.only(right: 20),
        decoration: BoxDecoration(
          color: scheme.error,
          borderRadius: BorderRadius.circular(12),
        ),
        child: Icon(Icons.delete, color: scheme.onError),
      ),
      onDismissed: (_) => onDelete(),
      child: Card(
        color: todo.completed
            ? scheme.surfaceContainerHighest.withValues(alpha: 0.5)
            : null,
        child: CheckboxListTile(
          value: todo.completed,
          onChanged: (_) => onToggle(),
          title: Text(
            todo.title,
            style: TextStyle(
              fontWeight: FontWeight.w700,
              decoration: todo.completed ? TextDecoration.lineThrough : null,
              color: todo.completed ? scheme.onSurfaceVariant : null,
            ),
          ),
          subtitle: Padding(
            padding: const EdgeInsets.only(top: 6),
            child: Wrap(
              spacing: 8,
              children: [
                _PriorityChip(priority: todo.priority),
                Chip(
                  label: Text(todo.lane),
                  padding: EdgeInsets.zero,
                  visualDensity: VisualDensity.compact,
                ),
              ],
            ),
          ),
          controlAffinity: ListTileControlAffinity.leading,
          contentPadding: const EdgeInsets.symmetric(
            horizontal: 18,
            vertical: 8,
          ),
        ),
      ),
    );
  }
}

class _PriorityChip extends StatelessWidget {
  const _PriorityChip({required this.priority});

  final String priority;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final (bgColor, fgColor) = switch (priority.toLowerCase()) {
      'high' => (scheme.errorContainer, scheme.onErrorContainer),
      'medium' => (scheme.secondaryContainer, scheme.onSecondaryContainer),
      _ => (scheme.tertiaryContainer, scheme.onTertiaryContainer),
    };

    return Chip(
      label: Text(priority),
      backgroundColor: bgColor,
      labelStyle: TextStyle(color: fgColor, fontWeight: FontWeight.bold),
      padding: EdgeInsets.zero,
      visualDensity: VisualDensity.compact,
      side: BorderSide.none,
    );
  }
}

class _EmptyTaskState extends StatelessWidget {
  const _EmptyTaskState();

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          children: [
            Icon(
              Icons.inbox_outlined,
              size: 40,
              color: Theme.of(context).colorScheme.primary,
            ),
            const SizedBox(height: 12),
            Text(
              'No tasks match the current filter.',
              style: Theme.of(context).textTheme.titleMedium,
              textAlign: TextAlign.center,
            ),
          ],
        ),
      ),
    );
  }
}
