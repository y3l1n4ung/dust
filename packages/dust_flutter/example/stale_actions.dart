import 'dart:async';

import 'package:dust_flutter/state.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

/// Search-as-you-type without a slow answer overwriting a newer one.
///
/// Each keystroke starts a search, and nothing guarantees the answers arrive
/// in order: "c" can take longer than "co". `runAction` gives every search the
/// same key, so starting one supersedes the last, and an answer for a
/// superseded search is dropped instead of emitted.
///
/// "No matches" is an effect rather than state: it is a one-off message, and
/// state would show it again on every rebuild.
///
/// In an app, `@ViewModel(state: SearchState)` generates the base this class
/// extends. It is extended by hand here so the example runs without
/// `dust build`.
///
/// ```shell
/// flutter run example/stale_actions.dart
/// ```
void main() {
  runApp(
    MaterialApp(
      home: SearchPage(
        viewModel: SearchViewModel(
          const SearchArgs(search: slowerForShorterQueries),
        ),
      ),
    ),
  );
}

/// A search whose short queries take longest, the order that breaks naive code.
Future<List<String>> slowerForShorterQueries(String query) async {
  const drinks = ['Cocoa', 'Coffee', 'Cola', 'Tea'];
  await Future<void>.delayed(Duration(milliseconds: 600 ~/ (query.length + 1)));
  return [
    for (final drink in drinks)
      if (drink.toLowerCase().startsWith(query.toLowerCase())) drink,
  ];
}

/// What the search page shows.
@immutable
final class SearchState {
  /// Creates a state.
  const SearchState(
      {this.query = '', this.results = const [], this.busy = false});

  /// The query the results belong to.
  final String query;

  /// Matching names.
  final List<String> results;

  /// Whether a search is running.
  final bool busy;

  @override
  bool operator ==(Object other) =>
      other is SearchState &&
      other.query == query &&
      other.busy == busy &&
      listEquals(other.results, results);

  @override
  int get hashCode => Object.hash(query, busy, Object.hashAll(results));
}

/// A search finished with nothing to show.
final class NoMatches {
  /// Creates the effect.
  const NoMatches(this.query);

  /// The query that matched nothing.
  final String query;
}

/// Dependencies of [SearchViewModel].
final class SearchArgs extends ViewModelArgs {
  /// Creates the args.
  const SearchArgs({required this.search, super.observer});

  /// Finds names starting with a query.
  final Future<List<String>> Function(String query) search;
}

/// Runs one search at a time, as far as the screen can tell.
final class SearchViewModel extends ViewModelBase<SearchState, SearchArgs> {
  /// Creates the view model.
  SearchViewModel(super.args) : super(initialState: const SearchState());

  static const Object _search = Object();

  /// Searches for [query], superseding any search still running.
  Future<void> search(String query) async {
    await runAction<List<String>>(
      _search,
      onStart: () =>
          emit(SearchState(query: query, results: state.results, busy: true)),
      run: () => args.search(query),
      onSuccess: (results) {
        emit(SearchState(query: query, results: results));
        if (results.isEmpty) emitEffect(NoMatches(query));
      },
    );
  }
}

/// A search field, its results, and a snackbar for an empty search.
class SearchPage extends StatefulWidget {
  /// Creates the page.
  const SearchPage({required this.viewModel, super.key});

  /// The view model this page owns.
  final SearchViewModel viewModel;

  @override
  State<SearchPage> createState() => _SearchPageState();
}

class _SearchPageState extends State<SearchPage> {
  late final StreamSubscription<Object> _effects;

  @override
  void initState() {
    super.initState();
    _effects = widget.viewModel.effects.listen((effect) {
      if (effect is NoMatches && mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Nothing starts with "${effect.query}".')),
        );
      }
    });
  }

  @override
  void dispose() {
    unawaited(_effects.cancel());
    widget.viewModel.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Drinks')),
      body: ValueListenableBuilder<SearchState>(
        valueListenable: widget.viewModel,
        builder: (context, state, _) => Column(
          children: [
            TextField(onChanged: widget.viewModel.search),
            if (state.busy) const LinearProgressIndicator(),
            Expanded(
              child: ListView(
                children: [
                  for (final name in state.results) ListTile(title: Text(name))
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
