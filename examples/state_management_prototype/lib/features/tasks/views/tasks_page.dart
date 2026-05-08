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

  @override
  Widget build(BuildContext context) {
    final viewModel = context.taskBoardViewModel;

    return ValueListenableBuilder<TaskBoardState>(
      valueListenable: viewModel,
      builder: (context, state, child) {
        if (_queryController.text != state.query) {
          _queryController.value = TextEditingValue(
            text: state.query,
            selection: TextSelection.collapsed(offset: state.query.length),
          );
        }

        return Padding(
          padding: const EdgeInsets.fromLTRB(20, 12, 20, 0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'Task board',
                style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                      fontWeight: FontWeight.w900,
                    ),
              ),
              const SizedBox(height: 8),
              Text(
                'Filter, search, and update generated Dust models in-place.',
                style: Theme.of(context).textTheme.bodyMedium,
              ),
              const SizedBox(height: 18),
              if (state.errorMessage != null)
                Container(
                  width: double.infinity,
                  margin: const EdgeInsets.only(bottom: 14),
                  padding:
                      const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
                  decoration: BoxDecoration(
                    color: Theme.of(context).colorScheme.errorContainer,
                    borderRadius: BorderRadius.circular(18),
                  ),
                  child: Text(
                    state.errorMessage!,
                    style: TextStyle(
                      color: Theme.of(context).colorScheme.onErrorContainer,
                    ),
                  ),
                ),
              TextField(
                controller: _queryController,
                onChanged: viewModel.setQuery,
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
                        selected: state.filter == filter,
                        onSelected: (_) => viewModel.setFilter(filter),
                      ),
                    )
                    .toList(),
              ),
              const SizedBox(height: 14),
              Expanded(
                child: RefreshIndicator(
                  onRefresh: viewModel.refresh,
                  child: state.visibleTodos.isEmpty
                      ? ListView(
                          children: const [
                            SizedBox(height: 120),
                            _EmptyTaskState(),
                          ],
                        )
                      : ListView.separated(
                          itemCount: state.visibleTodos.length,
                          separatorBuilder: (_, _) =>
                              const SizedBox(height: 12),
                          itemBuilder: (context, index) {
                            final todo = state.visibleTodos[index];
                            return _TodoTile(
                              todo: todo,
                              onToggle: () => viewModel.toggleTodo(todo.id),
                            );
                          },
                        ),
                ),
              ),
            ],
          ),
        );
      },
    );
  }
}

String _filterLabel(TodoFilter filter) => switch (filter) {
      TodoFilter.all => 'All',
      TodoFilter.open => 'Open',
      TodoFilter.done => 'Done',
    };

class _TodoTile extends StatelessWidget {
  const _TodoTile({required this.todo, required this.onToggle});

  final RemoteTodo todo;
  final VoidCallback onToggle;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;

    return Card(
      child: CheckboxListTile(
        value: todo.completed,
        onChanged: (_) => onToggle(),
        title: Text(
          todo.title,
          style: TextStyle(
            fontWeight: FontWeight.w700,
            decoration: todo.completed ? TextDecoration.lineThrough : null,
          ),
        ),
        subtitle: Padding(
          padding: const EdgeInsets.only(top: 6),
          child: Wrap(
            spacing: 8,
            children: [
              Chip(label: Text(todo.lane)),
              Chip(
                label: Text(todo.priority),
                backgroundColor: scheme.secondaryContainer,
              ),
            ],
          ),
        ),
        controlAffinity: ListTileControlAffinity.leading,
        contentPadding: const EdgeInsets.symmetric(horizontal: 18, vertical: 8),
      ),
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
