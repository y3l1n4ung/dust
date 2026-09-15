import 'package:dust_dart/fp.dart';
import 'package:dust_flutter/state.dart';
import 'package:flutter/material.dart';

/// Loading data that can fail, and keeping the failure's own type.
///
/// Data sources in a Dust app often return `Result`: a generated DAO answers
/// `Result<T, SqlxError>`. An async view model's `loadData` returns plain
/// data, and the base class turns a
/// thrown error into `AsyncFailure`. Throwing the `Err` value itself, rather
/// than calling `unwrap`, is what keeps `AsyncFailure.error` a `CatalogError`
/// the page can switch on instead of a `StateError` holding a string.
///
/// In an app, `@ViewModel(..., mode: ViewModelMode.async)` generates the base
/// this class extends, and the generated builder replaces the `switch` below.
/// Both are written by hand here so the example runs without `dust build`.
///
/// ```shell
/// flutter run example/async_view_model.dart
/// ```
void main() {
  runApp(
    MaterialApp(
      home: CatalogPage(
        viewModel: CatalogViewModel(
          CatalogArgs(repository: FlakyCatalogRepository()),
        ),
      ),
    ),
  );
}

/// Why the catalog could not be read.
sealed class CatalogError {
  const CatalogError();
}

/// The network is unreachable; trying again may work.
final class CatalogOffline extends CatalogError {
  /// Creates an offline error.
  const CatalogOffline();
}

/// The account may not read the catalog; trying again will not help.
final class CatalogForbidden extends CatalogError {
  /// Creates a forbidden error.
  const CatalogForbidden();
}

/// Where the catalog comes from.
abstract interface class CatalogRepository {
  /// Reads every product name.
  Future<Result<List<String>, CatalogError>> products();
}

/// A repository that is offline once, then answers.
final class FlakyCatalogRepository implements CatalogRepository {
  var _calls = 0;

  @override
  Future<Result<List<String>, CatalogError>> products() async {
    await Future<void>.delayed(const Duration(milliseconds: 300));
    _calls++;
    if (_calls == 1) return const Err(CatalogOffline());
    return Ok(['Tea', 'Coffee', if (_calls > 2) 'Cocoa']);
  }
}

/// Dependencies of [CatalogViewModel].
final class CatalogArgs extends ViewModelArgs {
  /// Creates the args.
  const CatalogArgs({required this.repository, super.observer});

  /// Source of the catalog.
  final CatalogRepository repository;
}

/// Loads the catalog into `AsyncState<List<String>>`.
final class CatalogViewModel
    extends AsyncViewModelBase<List<String>, CatalogArgs> {
  /// Creates the view model.
  CatalogViewModel(super.args);

  @override
  Future<List<String>> loadData() async {
    final products = await args.repository.products();
    return products.unwrapOrElse((error) => throw error);
  }
}

/// Shows each [AsyncState] the catalog moves through.
class CatalogPage extends StatefulWidget {
  /// Creates the page.
  const CatalogPage({required this.viewModel, super.key});

  /// The view model this page owns.
  final CatalogViewModel viewModel;

  @override
  State<CatalogPage> createState() => _CatalogPageState();
}

class _CatalogPageState extends State<CatalogPage> {
  @override
  void initState() {
    super.initState();
    widget.viewModel.load();
  }

  @override
  void dispose() {
    widget.viewModel.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final viewModel = widget.viewModel;
    return Scaffold(
      appBar: AppBar(
        title: const Text('Catalog'),
        actions: [
          IconButton(
            tooltip: 'Refresh',
            icon: const Icon(Icons.refresh),
            onPressed: viewModel.refresh,
          ),
        ],
      ),
      body: ValueListenableBuilder<AsyncState<List<String>>>(
        valueListenable: viewModel,
        builder: (context, state, _) => switch (state) {
          AsyncInitial() => const SizedBox.shrink(),
          // A refresh keeps what is already on screen.
          AsyncLoading(hasPreviousData: true, :final previousData?) =>
            _Products(previousData, refreshing: true),
          AsyncLoading() => const Center(child: CircularProgressIndicator()),
          AsyncData(:final data) => _Products(data),
          AsyncFailure(:final error) => _Failure(
              message: switch (error) {
                CatalogOffline() => 'You are offline.',
                CatalogForbidden() => 'You cannot see this catalog.',
                _ => 'Something went wrong.',
              },
              onRetry: error is CatalogOffline ? viewModel.retry : null,
            ),
        },
      ),
    );
  }
}

class _Products extends StatelessWidget {
  const _Products(this.products, {this.refreshing = false});

  final List<String> products;
  final bool refreshing;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        if (refreshing) const LinearProgressIndicator(),
        Expanded(
          child: ListView(
            children: [
              for (final name in products) ListTile(title: Text(name))
            ],
          ),
        ),
      ],
    );
  }
}

class _Failure extends StatelessWidget {
  const _Failure({required this.message, required this.onRetry});

  final String message;
  final VoidCallback? onRetry;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(message),
          if (onRetry != null)
            TextButton(onPressed: onRetry, child: const Text('Retry')),
        ],
      ),
    );
  }
}
