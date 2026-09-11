import 'package:dust_flutter/state.dart';
import 'package:flutter_test/flutter_test.dart';

/// A state type with its own equality, which is the whole point: an async view
/// model has to honour it the way a synchronous one already did.
final class Page {
  const Page(this.title);

  final String title;

  @override
  bool operator ==(Object other) => other is Page && other.title == title;

  @override
  int get hashCode => title.hashCode;
}

final class Args extends ViewModelArgs {
  const Args();
}

final class PageViewModel extends AsyncViewModelBase<Page, Args> {
  PageViewModel(this.pages) : super(const Args());

  final List<Page> pages;
  int loadCount = 0;

  @override
  Future<Page> loadData() async => pages[loadCount++];

  /// `emit` is protected, and the guard it runs is what this file is about.
  void emitState(AsyncState<Page> next) => emit(next);
}

void main() {
  group('equality', () {
    test('data states compare by their value', () {
      expect(const AsyncData(Page('home')), const AsyncData(Page('home')));
      expect(
        const AsyncData(Page('home')).hashCode,
        const AsyncData(Page('home')).hashCode,
      );
      expect(
        const AsyncData(Page('home')),
        isNot(const AsyncData(Page('about'))),
      );
    });

    test('initial states of one type are interchangeable', () {
      expect(const AsyncInitial<Page>(), const AsyncInitial<Page>());
      expect(const AsyncInitial<Page>(), isNot(const AsyncInitial<String>()));
    });

    test('loading states compare on the data they preserve', () {
      expect(const AsyncLoading<Page>(), const AsyncLoading<Page>());
      expect(
        const AsyncLoading<Page>(
          previousData: Page('home'),
          hasPreviousData: true,
        ),
        const AsyncLoading<Page>(
          previousData: Page('home'),
          hasPreviousData: true,
        ),
      );
      expect(
        const AsyncLoading<Page>(
          previousData: Page('home'),
          hasPreviousData: true,
        ),
        isNot(const AsyncLoading<Page>()),
      );
    });

    test('a failure that retried the same way is still a new failure', () {
      final error = StateError('offline');
      final first = StackTrace.current;
      final second = StackTrace.current;

      expect(
        AsyncFailure<Page>(error, first),
        AsyncFailure<Page>(error, first),
      );
      expect(
        AsyncFailure<Page>(error, first),
        isNot(AsyncFailure<Page>(error, second)),
      );
    });

    test('a state that differs only in its type argument is not equal', () {
      expect(const AsyncData<Object>(1), isNot(const AsyncData<int>(1)));
    });
  });

  group('notification', () {
    test('emitting an equal state again does not notify', () async {
      // `ViewModelBase` skips notifying when the next state equals the current
      // one. Before these operators the wrapper compared by identity, so this
      // notified twice even though the view model had not changed.
      final viewModel = PageViewModel([const Page('home')]);
      addTearDown(viewModel.dispose);

      var notifications = 0;
      viewModel.addListener(() => notifications++);

      viewModel.emitState(const AsyncData(Page('home')));
      viewModel.emitState(const AsyncData(Page('home')));

      expect(notifications, 1);
      expect(viewModel.state, const AsyncData(Page('home')));
    });

    test('emitting a different value still notifies', () async {
      final viewModel = PageViewModel([const Page('home')]);
      addTearDown(viewModel.dispose);

      var notifications = 0;
      viewModel.addListener(() => notifications++);

      viewModel.emitState(const AsyncData(Page('home')));
      viewModel.emitState(const AsyncData(Page('about')));

      expect(notifications, 2);
    });

    test('a load still reports its loading state', () async {
      // Equality suppresses repeats, not the lifecycle: reloading the same data
      // still passes through loading, so listeners see both transitions.
      final viewModel = PageViewModel([const Page('home'), const Page('home')]);
      addTearDown(viewModel.dispose);

      await viewModel.load();
      final seen = <AsyncState<Page>>[];
      viewModel.addListener(() => seen.add(viewModel.state));

      await viewModel.load();

      expect(seen, [const AsyncLoading<Page>(), const AsyncData(Page('home'))]);
    });
  });

  group('hash codes', () {
    test('equal states hash equally, and each variant hashes distinctly', () {
      // Every variant's `hashCode`, so a map or set keyed by state behaves.
      expect(
        const AsyncInitial<Page>().hashCode,
        const AsyncInitial<Page>().hashCode,
      );
      expect(
        const AsyncLoading<Page>(
          previousData: Page('home'),
          hasPreviousData: true,
        ).hashCode,
        const AsyncLoading<Page>(
          previousData: Page('home'),
          hasPreviousData: true,
        ).hashCode,
      );

      final error = StateError('offline');
      final trace = StackTrace.current;
      expect(
        AsyncFailure<Page>(error, trace).hashCode,
        AsyncFailure<Page>(error, trace).hashCode,
      );

      // Added one at a time: the repeated element is the point of the
      // assertion, and a set literal holding one does not get past the
      // analyzer.
      final states = <AsyncState<Page>>{};
      for (final state in <AsyncState<Page>>[
        const AsyncInitial<Page>(),
        const AsyncLoading<Page>(),
        const AsyncData(Page('home')),
        AsyncFailure<Page>(error, trace),
        const AsyncData(Page('home')),
      ]) {
        states.add(state);
      }
      expect(states, hasLength(4), reason: 'the repeated data state collapses');
    });
  });

  group('predicates', () {
    test('each variant answers the lifecycle questions', () {
      const initial = AsyncInitial<Page>();
      expect(initial.isLoading, isFalse);
      expect(initial.hasData, isFalse);
      expect(initial.hasPreviousData, isFalse);
      expect(initial.isRefreshing, isFalse);
      expect(initial.data, isNull);
      expect(initial.previousData, isNull);
      expect(initial.error, isNull);
      expect(initial.stackTrace, isNull);

      const loading = AsyncLoading<Page>();
      expect(loading.isLoading, isTrue);
      expect(loading.isRefreshing, isFalse);
      expect(loading.hasData, isFalse);

      const refreshing = AsyncLoading<Page>(
        previousData: Page('home'),
        hasPreviousData: true,
      );
      expect(refreshing.isRefreshing, isTrue);
      expect(refreshing.hasData, isTrue);
      expect(refreshing.data, const Page('home'));

      const data = AsyncData(Page('home'));
      expect(data.isLoading, isFalse);
      expect(data.hasData, isTrue);

      final stale = AsyncFailure<Page>(
        StateError('offline'),
        StackTrace.empty,
        previousData: const Page('home'),
        hasPreviousData: true,
      );
      expect(stale.isLoading, isFalse);
      expect(stale.hasData, isTrue);
      expect(stale.data, const Page('home'));
      expect(stale.error, isA<StateError>());
      expect(stale.stackTrace, StackTrace.empty);
    });
  });

  group('toString', () {
    test('names the state and what it carries', () {
      expect(const AsyncInitial<Page>().toString(), 'AsyncInitial<Page>()');
      expect(const AsyncLoading<Page>().toString(), 'AsyncLoading<Page>()');
      expect(
        const AsyncData(Page('home')).toString(),
        startsWith('AsyncData<Page>('),
      );
      expect(
        const AsyncLoading<Page>(
          previousData: Page('home'),
          hasPreviousData: true,
        ).toString(),
        startsWith('AsyncLoading<Page>(refreshing:'),
      );
      expect(
        AsyncFailure<Page>(StateError('offline'), StackTrace.empty).toString(),
        startsWith('AsyncFailure<Page>('),
      );
      expect(
        AsyncFailure<Page>(
          StateError('offline'),
          StackTrace.empty,
          previousData: const Page('home'),
          hasPreviousData: true,
        ).toString(),
        contains('stale:'),
      );
    });
  });
}
