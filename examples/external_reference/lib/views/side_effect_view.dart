import 'package:flutter/material.dart';

import '../view_models/side_effect_view_model.dart';

/// UI Side Effects Section - demonstrates one-time UI actions.
class _UiSideEffectsSection extends StatefulWidget {
  const _UiSideEffectsSection();

  @override
  State<_UiSideEffectsSection> createState() => _UiSideEffectsSectionState();
}

class _UiSideEffectsSectionState extends State<_UiSideEffectsSection> {
  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    // Set up listener for UI side effects (snackbars, dialogs, navigation)
    context.listenOnSideEffectViewModel(
      (previous, current) {
        // Show snackbar on fetch success
        if (previous.fetchStatus != AsyncStatus.success &&
            current.fetchStatus == AsyncStatus.success) {
          _showSnackBar(
            'Data fetched successfully!',
            Colors.green,
            Icons.check_circle,
          );
        }

        // Show snackbar on fetch error
        if (previous.fetchStatus != AsyncStatus.error &&
            current.fetchStatus == AsyncStatus.error) {
          _showSnackBar(
            'Fetch failed: ${current.fetchError}',
            Colors.red,
            Icons.error,
          );
        }

        // Show snackbar when timer completes
        if (previous.timerSeconds > 0 &&
            current.timerSeconds == 0 &&
            !current.isTimerRunning &&
            previous.isTimerRunning) {
          _showSnackBar(
            'Timer completed!',
            Colors.orange,
            Icons.alarm,
          );
        }
      },
    );
  }

  void _showSnackBar(String message, Color color, IconData icon) {
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Row(
          children: [
            Icon(icon, color: Colors.white, size: 20),
            const SizedBox(width: 8),
            Expanded(child: Text(message)),
          ],
        ),
        backgroundColor: color,
        behavior: SnackBarBehavior.floating,
        duration: const Duration(seconds: 2),
      ),
    );
  }

  void _showConfirmDialog() {
    showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('Confirm Action'),
        content: const Text(
          'This demonstrates a dialog triggered as a UI side effect. '
          'Would you like to fetch data?',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(false),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () {
              Navigator.of(dialogContext).pop(true);
              context.readSideEffectViewModel().fetchData();
            },
            child: const Text('Fetch'),
          ),
        ],
      ),
    );
  }

  @override
  void dispose() {
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.notifications_active, size: 20),
                const SizedBox(width: 8),
                Text(
                  'UI Side Effects',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              'One-time UI actions like snackbars, dialogs, and navigation '
              'triggered by state changes using listeners.',
              style: Theme.of(context).textTheme.bodySmall,
            ),
            const SizedBox(height: 16),
            // Info box
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Colors.blue.withValues(alpha: 0.1),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(
                  color: Colors.blue.withValues(alpha: 0.3),
                ),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      const Icon(Icons.info, size: 16, color: Colors.blue),
                      const SizedBox(width: 8),
                      Text(
                        'How it works',
                        style: Theme.of(context).textTheme.labelLarge?.copyWith(
                              color: Colors.blue,
                            ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 8),
                  const Text(
                    '• Uses listenOnSideEffectViewModel() in didChangeDependencies\n'
                    '• Compares previous vs current state to detect changes\n'
                    '• Triggers snackbars/dialogs only on state transitions\n'
                    '• Subscription is cancelled in dispose()',
                    style: TextStyle(fontSize: 12),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 16),
            // Demo buttons
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                ElevatedButton.icon(
                  onPressed: _showConfirmDialog,
                  icon: const Icon(Icons.question_answer, size: 18),
                  label: const Text('Show Dialog'),
                ),
                OutlinedButton.icon(
                  onPressed: () {
                    _showSnackBar(
                      'Manual snackbar triggered!',
                      Colors.purple,
                      Icons.touch_app,
                    );
                  },
                  icon: const Icon(Icons.message, size: 18),
                  label: const Text('Test Snackbar'),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Text(
              'Try the Fetch Data button below - snackbars will appear automatically '
              'on success or error!',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    fontStyle: FontStyle.italic,
                  ),
            ),
          ],
        ),
      ),
    );
  }
}

/// Side Effect View - demonstrates async operations and streaming.
class SideEffectView extends StatelessWidget {
  const SideEffectView({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Side Effects & Streaming'),
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const _UiSideEffectsSection(),
            const SizedBox(height: 24),
            const _AsyncOperationSection(),
            const SizedBox(height: 24),
            const _DebouncedSearchSection(),
            const SizedBox(height: 24),
            const _StreamingSection(),
            const SizedBox(height: 24),
            const _TimerSection(),
          ],
        ),
      ),
    );
  }
}

/// Async operation with retry demo.
class _AsyncOperationSection extends StatelessWidget {
  const _AsyncOperationSection();

  @override
  Widget build(BuildContext context) {
    final vm = context.sideEffectViewModel;
    final fetchStatus = context.selectSideEffectViewModel((s) => s.fetchStatus);
    final fetchResult = context.selectSideEffectViewModel((s) => s.fetchResult);
    final fetchError = context.selectSideEffectViewModel((s) => s.fetchError);
    final retryCount = context.selectSideEffectViewModel((s) => s.retryCount);

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.cloud_download, size: 20),
                const SizedBox(width: 8),
                Text(
                  'Async Operations',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              'Demonstrates async API calls with error handling and retry.',
              style: Theme.of(context).textTheme.bodySmall,
            ),
            const SizedBox(height: 16),
            // Status display
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: _getStatusColor(fetchStatus).withValues(alpha: 0.1),
                borderRadius: BorderRadius.circular(8),
              ),
              child: Row(
                children: [
                  if (fetchStatus == AsyncStatus.loading)
                    const SizedBox(
                      width: 16,
                      height: 16,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  else
                    Icon(
                      _getStatusIcon(fetchStatus),
                      size: 16,
                      color: _getStatusColor(fetchStatus),
                    ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      fetchError ?? fetchResult ?? 'Ready to fetch',
                      style: TextStyle(
                        fontSize: 12,
                        color: _getStatusColor(fetchStatus),
                      ),
                    ),
                  ),
                ],
              ),
            ),
            if (retryCount > 0) ...[
              const SizedBox(height: 8),
              Text(
                'Retry count: $retryCount',
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ],
            const SizedBox(height: 16),
            // Actions
            Wrap(
              spacing: 8,
              children: [
                ElevatedButton.icon(
                  onPressed:
                      fetchStatus == AsyncStatus.loading ? null : vm.fetchData,
                  icon: const Icon(Icons.download, size: 18),
                  label: const Text('Fetch Data'),
                ),
                if (fetchStatus == AsyncStatus.error)
                  OutlinedButton.icon(
                    onPressed: vm.retryWithBackoff,
                    icon: const Icon(Icons.refresh, size: 18),
                    label: const Text('Retry'),
                  ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Color _getStatusColor(AsyncStatus status) {
    switch (status) {
      case AsyncStatus.idle:
        return Colors.grey;
      case AsyncStatus.loading:
        return Colors.blue;
      case AsyncStatus.success:
        return Colors.green;
      case AsyncStatus.error:
        return Colors.red;
    }
  }

  IconData _getStatusIcon(AsyncStatus status) {
    switch (status) {
      case AsyncStatus.idle:
        return Icons.hourglass_empty;
      case AsyncStatus.loading:
        return Icons.sync;
      case AsyncStatus.success:
        return Icons.check_circle;
      case AsyncStatus.error:
        return Icons.error;
    }
  }
}

/// Debounced search demo.
class _DebouncedSearchSection extends StatelessWidget {
  const _DebouncedSearchSection();

  @override
  Widget build(BuildContext context) {
    final vm = context.sideEffectViewModel;
    final searchResults =
        context.selectSideEffectViewModel((s) => s.searchResults);
    final isSearching = context.selectSideEffectViewModel((s) => s.isSearching);

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.search, size: 20),
                const SizedBox(width: 8),
                Text(
                  'Debounced Search',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              'Waits 300ms after typing stops before searching.',
              style: Theme.of(context).textTheme.bodySmall,
            ),
            const SizedBox(height: 16),
            TextField(
              decoration: InputDecoration(
                hintText: 'Type to search...',
                prefixIcon: const Icon(Icons.search),
                suffixIcon: isSearching
                    ? const Padding(
                        padding: EdgeInsets.all(12),
                        child: SizedBox(
                          width: 20,
                          height: 20,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        ),
                      )
                    : null,
                border: const OutlineInputBorder(),
              ),
              onChanged: vm.search,
            ),
            if (searchResults.isNotEmpty) ...[
              const SizedBox(height: 12),
              Container(
                decoration: BoxDecoration(
                  border: Border.all(
                    color: Theme.of(context).colorScheme.outlineVariant,
                  ),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: ListView.separated(
                  shrinkWrap: true,
                  physics: const NeverScrollableScrollPhysics(),
                  itemCount: searchResults.length,
                  separatorBuilder: (_, __) => const Divider(height: 1),
                  itemBuilder: (context, index) => ListTile(
                    dense: true,
                    leading: const Icon(Icons.article, size: 18),
                    title: Text(searchResults[index]),
                  ),
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}

/// Stock price streaming demo.
class _StreamingSection extends StatelessWidget {
  const _StreamingSection();

  @override
  Widget build(BuildContext context) {
    final vm = context.sideEffectViewModel;
    final stockPrices = context.selectSideEffectViewModel((s) => s.stockPrices);
    final isStreamActive =
        context.selectSideEffectViewModel((s) => s.isStreamActive);
    final tickCount = context.selectSideEffectViewModel((s) => s.tickCount);

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(
                  Icons.stream,
                  size: 20,
                  color: isStreamActive ? Colors.green : null,
                ),
                const SizedBox(width: 8),
                Text(
                  'Stock Price Stream',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                const Spacer(),
                if (isStreamActive)
                  Container(
                    padding:
                        const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                    decoration: BoxDecoration(
                      color: Colors.green.withValues(alpha: 0.1),
                      borderRadius: BorderRadius.circular(12),
                    ),
                    child: Text(
                      'LIVE • $tickCount ticks',
                      style: const TextStyle(
                        fontSize: 10,
                        color: Colors.green,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                  ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              'Real-time stream subscription with automatic updates.',
              style: Theme.of(context).textTheme.bodySmall,
            ),
            const SizedBox(height: 16),
            // Stock prices
            if (stockPrices.isEmpty)
              Container(
                padding: const EdgeInsets.all(24),
                alignment: Alignment.center,
                child: Text(
                  'Start stream to see live prices',
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              )
            else
              SizedBox(
                height: 120,
                child: ListView.builder(
                  scrollDirection: Axis.horizontal,
                  itemCount: stockPrices.length,
                  itemBuilder: (context, index) {
                    final price = stockPrices[index];
                    return Container(
                      width: 100,
                      margin: const EdgeInsets.only(right: 8),
                      padding: const EdgeInsets.all(12),
                      decoration: BoxDecoration(
                        color: price.isPositive
                            ? Colors.green.withValues(alpha: 0.1)
                            : Colors.red.withValues(alpha: 0.1),
                        borderRadius: BorderRadius.circular(8),
                        border: Border.all(
                          color: price.isPositive
                              ? Colors.green.withValues(alpha: 0.3)
                              : Colors.red.withValues(alpha: 0.3),
                        ),
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            price.symbol,
                            style: const TextStyle(
                              fontWeight: FontWeight.bold,
                              fontSize: 12,
                            ),
                          ),
                          const Spacer(),
                          Text(
                            '\$${price.price.toStringAsFixed(2)}',
                            style: const TextStyle(
                              fontSize: 16,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                          Row(
                            children: [
                              Icon(
                                price.isPositive
                                    ? Icons.arrow_upward
                                    : Icons.arrow_downward,
                                size: 12,
                                color: price.isPositive
                                    ? Colors.green
                                    : Colors.red,
                              ),
                              Text(
                                price.change.toStringAsFixed(2),
                                style: TextStyle(
                                  fontSize: 10,
                                  color: price.isPositive
                                      ? Colors.green
                                      : Colors.red,
                                ),
                              ),
                            ],
                          ),
                        ],
                      ),
                    );
                  },
                ),
              ),
            const SizedBox(height: 16),
            Row(
              children: [
                ElevatedButton.icon(
                  onPressed: isStreamActive ? null : vm.startStockStream,
                  icon: const Icon(Icons.play_arrow, size: 18),
                  label: const Text('Start'),
                ),
                const SizedBox(width: 8),
                OutlinedButton.icon(
                  onPressed: isStreamActive ? vm.stopStockStream : null,
                  icon: const Icon(Icons.stop, size: 18),
                  label: const Text('Stop'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

/// Countdown timer demo.
class _TimerSection extends StatelessWidget {
  const _TimerSection();

  @override
  Widget build(BuildContext context) {
    final vm = context.sideEffectViewModel;
    final timerSeconds =
        context.selectSideEffectViewModel((s) => s.timerSeconds);
    final isTimerRunning =
        context.selectSideEffectViewModel((s) => s.isTimerRunning);

    final minutes = timerSeconds ~/ 60;
    final seconds = timerSeconds % 60;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.timer, size: 20),
                const SizedBox(width: 8),
                Text(
                  'Countdown Timer',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              'Timer with pause/resume using periodic Timer.',
              style: Theme.of(context).textTheme.bodySmall,
            ),
            const SizedBox(height: 24),
            // Timer display
            Center(
              child: Container(
                padding:
                    const EdgeInsets.symmetric(horizontal: 32, vertical: 16),
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.surfaceContainerHighest,
                  borderRadius: BorderRadius.circular(16),
                ),
                child: Text(
                  '${minutes.toString().padLeft(2, '0')}:${seconds.toString().padLeft(2, '0')}',
                  style: const TextStyle(
                    fontSize: 48,
                    fontWeight: FontWeight.bold,
                    fontFamily: 'monospace',
                  ),
                ),
              ),
            ),
            const SizedBox(height: 24),
            // Preset buttons
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                for (final preset in [10, 30, 60, 120])
                  ActionChip(
                    label: Text('${preset}s'),
                    onPressed:
                        isTimerRunning ? null : () => vm.startTimer(preset),
                  ),
              ],
            ),
            const SizedBox(height: 16),
            // Controls
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                if (!isTimerRunning && timerSeconds > 0)
                  ElevatedButton.icon(
                    onPressed: vm.resumeTimer,
                    icon: const Icon(Icons.play_arrow),
                    label: const Text('Resume'),
                  )
                else if (isTimerRunning)
                  ElevatedButton.icon(
                    onPressed: vm.pauseTimer,
                    icon: const Icon(Icons.pause),
                    label: const Text('Pause'),
                  ),
                const SizedBox(width: 8),
                OutlinedButton.icon(
                  onPressed: timerSeconds > 0 ? vm.resetTimer : null,
                  icon: const Icon(Icons.refresh),
                  label: const Text('Reset'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
