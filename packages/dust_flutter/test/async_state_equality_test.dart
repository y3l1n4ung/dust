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
    });
  });
}
