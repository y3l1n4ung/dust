import 'package:ease_example/main.dart' show provders;
import 'package:ease_state_helper/ease_state_helper.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:ease_example/view_models/counter_view_model.dart';
import 'package:ease_example/view_models/cart_view_model.dart';
import 'package:ease_example/view_models/todo_view_model.dart';
import 'package:ease_example/view_models/local_form_view_model.dart';
import 'package:ease_example/models/product.dart';

void main() {
  group('CounterViewModel Provider', () {
    testWidgets('provides state to descendants', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: CounterViewModelProvider(
            child: Builder(
              builder: (context) {
                final counter = context.counterViewModel;
                return Text('${counter.state}');
              },
            ),
          ),
        ),
      );

      expect(find.text('0'), findsOneWidget);
    });

    testWidgets('watch pattern rebuilds on state change', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: CounterViewModelProvider(
            child: Builder(
              builder: (context) {
                final counter = context.counterViewModel;
                return Column(
                  children: [
                    Text('Count: ${counter.state}'),
                    ElevatedButton(
                      onPressed: counter.increment,
                      child: const Text('Inc'),
                    ),
                  ],
                );
              },
            ),
          ),
        ),
      );

      expect(find.text('Count: 0'), findsOneWidget);

      await tester.tap(find.text('Inc'));
      await tester.pump();

      expect(find.text('Count: 1'), findsOneWidget);
    });

    testWidgets('read pattern does not rebuild widget', (tester) async {
      var buildCount = 0;

      await tester.pumpWidget(
        MaterialApp(
          home: CounterViewModelProvider(
            child: Builder(
              builder: (context) {
                buildCount++;
                return ElevatedButton(
                  onPressed: () => context.readCounterViewModel().increment(),
                  child: const Text('Inc'),
                );
              },
            ),
          ),
        ),
      );

      expect(buildCount, 1);

      await tester.tap(find.text('Inc'));
      await tester.pump();

      // Should NOT rebuild since we used read
      expect(buildCount, 1);
    });

    testWidgets('throws StateError when no provider found', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              try {
                context.counterViewModel;
                return const Text('Found');
              } on StateError catch (e) {
                return Text('Error: ${e.message.split('\n').first}');
              }
            },
          ),
        ),
      );

      expect(find.textContaining('No CounterViewModel found'), findsOneWidget);
    });

    testWidgets('multiple widgets watch same state', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: CounterViewModelProvider(
            child: Column(
              children: [
                Builder(
                  builder: (context) =>
                      Text('A:${context.counterViewModel.state}'),
                ),
                Builder(
                  builder: (context) =>
                      Text('B:${context.counterViewModel.state}'),
                ),
                Builder(
                  builder: (context) => ElevatedButton(
                    onPressed: context.readCounterViewModel().increment,
                    child: const Text('Inc'),
                  ),
                ),
              ],
            ),
          ),
        ),
      );

      expect(find.text('A:0'), findsOneWidget);
      expect(find.text('B:0'), findsOneWidget);

      await tester.tap(find.text('Inc'));
      await tester.pump();

      expect(find.text('A:1'), findsOneWidget);
      expect(find.text('B:1'), findsOneWidget);
    });

    testWidgets('increment and decrement work', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: CounterViewModelProvider(
            child: Builder(
              builder: (context) {
                final counter = context.counterViewModel;
                return Column(
                  children: [
                    Text('${counter.state}'),
                    ElevatedButton(
                        onPressed: counter.increment, child: const Text('+')),
                    ElevatedButton(
                        onPressed: counter.decrement, child: const Text('-')),
                  ],
                );
              },
            ),
          ),
        ),
      );

      expect(find.text('0'), findsOneWidget);

      await tester.tap(find.text('+'));
      await tester.pump();
      expect(find.text('1'), findsOneWidget);

      await tester.tap(find.text('+'));
      await tester.pump();
      expect(find.text('2'), findsOneWidget);

      await tester.tap(find.text('-'));
      await tester.pump();
      expect(find.text('1'), findsOneWidget);
    });
  });

  group('Ease Root Widget', () {
    testWidgets('provides all states', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: EaseScope(
            providers: provders,
            child: Builder(
              builder: (context) {
                final counter = context.counterViewModel;
                final cart = context.cartViewModel;
                final todo = context.todoViewModel;

                return Column(
                  children: [
                    Text('Counter: ${counter.state}'),
                    Text('Cart: ${cart.state.items.length}'),
                    Text('Todo: ${todo.state.length}'),
                  ],
                );
              },
            ),
          ),
        ),
      );

      expect(find.text('Counter: 0'), findsOneWidget);
      expect(find.text('Cart: 0'), findsOneWidget);
      expect(find.text('Todo: 0'), findsOneWidget);
    });
  });

  group('CartViewModel', () {
    testWidgets('addToCart adds new item', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: CartViewModelProvider(
            child: Builder(
              builder: (context) {
                final cart = context.cartViewModel;
                return Column(
                  children: [
                    Text('Items: ${cart.state.items.length}'),
                    ElevatedButton(
                      onPressed: () => cart.addToCart(
                        const Product(
                          id: '1',
                          name: 'Test',
                          description: '',
                          price: 10.0,
                          imageUrl: '',
                        ),
                      ),
                      child: const Text('Add'),
                    ),
                  ],
                );
              },
            ),
          ),
        ),
      );

      expect(find.text('Items: 0'), findsOneWidget);

      await tester.tap(find.text('Add'));
      await tester.pump();

      expect(find.text('Items: 1'), findsOneWidget);
    });

    testWidgets('addToCart increments quantity for existing item',
        (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: CartViewModelProvider(
            child: Builder(
              builder: (context) {
                final cart = context.cartViewModel;
                return Column(
                  children: [
                    Text('Items: ${cart.state.items.length}'),
                    Text('Count: ${cart.state.itemCount}'),
                    ElevatedButton(
                      onPressed: () => cart.addToCart(
                        const Product(
                          id: '1',
                          name: 'Test',
                          description: '',
                          price: 10.0,
                          imageUrl: '',
                        ),
                      ),
                      child: const Text('Add'),
                    ),
                  ],
                );
              },
            ),
          ),
        ),
      );

      await tester.tap(find.text('Add'));
      await tester.pump();
      expect(find.text('Items: 1'), findsOneWidget);
      expect(find.text('Count: 1'), findsOneWidget);

      await tester.tap(find.text('Add'));
      await tester.pump();
      expect(find.text('Items: 1'), findsOneWidget); // Still 1 item
      expect(find.text('Count: 2'), findsOneWidget); // But quantity is 2
    });

    testWidgets('computed properties work', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: CartViewModelProvider(
            child: Builder(
              builder: (context) {
                final cart = context.cartViewModel;

                // Add items on first build
                if (cart.state.isEmpty) {
                  WidgetsBinding.instance.addPostFrameCallback((_) {
                    cart.addToCart(const Product(
                      id: '1',
                      name: 'A',
                      description: '',
                      price: 100.0,
                      imageUrl: '',
                    ));
                  });
                }

                return Column(
                  children: [
                    Text('Subtotal: ${cart.state.subtotal}'),
                    Text('Tax: ${cart.state.tax}'),
                    Text('Total: ${cart.state.total}'),
                    Text('Empty: ${cart.state.isEmpty}'),
                  ],
                );
              },
            ),
          ),
        ),
      );

      expect(find.text('Subtotal: 0.0'), findsOneWidget);
      expect(find.text('Empty: true'), findsOneWidget);

      await tester.pump();

      expect(find.text('Subtotal: 100.0'), findsOneWidget);
      expect(find.text('Tax: 10.0'), findsOneWidget);
      expect(find.text('Total: 110.0'), findsOneWidget);
      expect(find.text('Empty: false'), findsOneWidget);
    });

    testWidgets('removeFromCart removes item', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: CartViewModelProvider(
            child: Builder(
              builder: (context) {
                final cart = context.cartViewModel;
                return Column(
                  children: [
                    Text('Items: ${cart.state.items.length}'),
                    ElevatedButton(
                      onPressed: () => cart.addToCart(const Product(
                        id: '1',
                        name: 'Test',
                        description: '',
                        price: 10.0,
                        imageUrl: '',
                      )),
                      child: const Text('Add'),
                    ),
                    ElevatedButton(
                      onPressed: () => cart.removeFromCart('1'),
                      child: const Text('Remove'),
                    ),
                  ],
                );
              },
            ),
          ),
        ),
      );

      await tester.tap(find.text('Add'));
      await tester.pump();
      expect(find.text('Items: 1'), findsOneWidget);

      await tester.tap(find.text('Remove'));
      await tester.pump();
      expect(find.text('Items: 0'), findsOneWidget);
    });

    testWidgets('clearCart clears all items', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: CartViewModelProvider(
            child: Builder(
              builder: (context) {
                final cart = context.cartViewModel;
                return Column(
                  children: [
                    Text('Items: ${cart.state.items.length}'),
                    ElevatedButton(
                      onPressed: () {
                        cart.addToCart(const Product(
                          id: '1',
                          name: 'A',
                          description: '',
                          price: 10.0,
                          imageUrl: '',
                        ));
                        cart.addToCart(const Product(
                          id: '2',
                          name: 'B',
                          description: '',
                          price: 20.0,
                          imageUrl: '',
                        ));
                      },
                      child: const Text('Add'),
                    ),
                    ElevatedButton(
                      onPressed: cart.clearCart,
                      child: const Text('Clear'),
                    ),
                  ],
                );
              },
            ),
          ),
        ),
      );

      await tester.tap(find.text('Add'));
      await tester.pump();
      expect(find.text('Items: 2'), findsOneWidget);

      await tester.tap(find.text('Clear'));
      await tester.pump();
      expect(find.text('Items: 0'), findsOneWidget);
    });
  });

  group('TodoViewModel', () {
    testWidgets('add and remove todos', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: TodoViewModelProvider(
            child: Builder(
              builder: (context) {
                final todo = context.todoViewModel;
                return Column(
                  children: [
                    Text('Count: ${todo.state.length}'),
                    ElevatedButton(
                      onPressed: () => todo.add('Test Todo'),
                      child: const Text('Add'),
                    ),
                    ElevatedButton(
                      onPressed: () {
                        if (todo.state.isNotEmpty) {
                          todo.remove(todo.state.first.id);
                        }
                      },
                      child: const Text('Remove'),
                    ),
                  ],
                );
              },
            ),
          ),
        ),
      );

      expect(find.text('Count: 0'), findsOneWidget);

      await tester.tap(find.text('Add'));
      await tester.pump();
      expect(find.text('Count: 1'), findsOneWidget);

      await tester.tap(find.text('Add'));
      await tester.pump();
      expect(find.text('Count: 2'), findsOneWidget);

      await tester.tap(find.text('Remove'));
      await tester.pump();
      expect(find.text('Count: 1'), findsOneWidget);
    });

    testWidgets('toggle todo completed', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: TodoViewModelProvider(
            child: Builder(
              builder: (context) {
                final todo = context.todoViewModel;
                final completed =
                    todo.state.isNotEmpty ? todo.state.first.completed : false;
                return Column(
                  children: [
                    Text('Completed: $completed'),
                    ElevatedButton(
                      onPressed: () => todo.add('Test'),
                      child: const Text('Add'),
                    ),
                    ElevatedButton(
                      onPressed: () {
                        if (todo.state.isNotEmpty) {
                          todo.toggle(todo.state.first.id);
                        }
                      },
                      child: const Text('Toggle'),
                    ),
                  ],
                );
              },
            ),
          ),
        ),
      );

      await tester.tap(find.text('Add'));
      await tester.pump();
      expect(find.text('Completed: false'), findsOneWidget);

      await tester.tap(find.text('Toggle'));
      await tester.pump();
      expect(find.text('Completed: true'), findsOneWidget);

      await tester.tap(find.text('Toggle'));
      await tester.pump();
      expect(find.text('Completed: false'), findsOneWidget);
    });

    testWidgets('computed getters work', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: TodoViewModelProvider(
            child: Builder(
              builder: (context) {
                final todo = context.todoViewModel;
                return Column(
                  children: [
                    Text('Total: ${todo.total}'),
                    Text('Completed: ${todo.completedCount}'),
                    Text('Pending: ${todo.pendingCount}'),
                    ElevatedButton(
                      onPressed: () => todo.add('Test'),
                      child: const Text('Add'),
                    ),
                    ElevatedButton(
                      onPressed: () {
                        if (todo.state.isNotEmpty) {
                          todo.toggle(todo.state.first.id);
                        }
                      },
                      child: const Text('Toggle'),
                    ),
                  ],
                );
              },
            ),
          ),
        ),
      );

      expect(find.text('Total: 0'), findsOneWidget);

      await tester.tap(find.text('Add'));
      await tester.tap(find.text('Add'));
      await tester.pump();

      expect(find.text('Total: 2'), findsOneWidget);
      expect(find.text('Completed: 0'), findsOneWidget);
      expect(find.text('Pending: 2'), findsOneWidget);

      await tester.tap(find.text('Toggle'));
      await tester.pump();

      expect(find.text('Total: 2'), findsOneWidget);
      expect(find.text('Completed: 1'), findsOneWidget);
      expect(find.text('Pending: 1'), findsOneWidget);
    });
  });

  group('Nested Local Provider', () {
    testWidgets('local provider does not rebuild when parent rebuilds',
        (tester) async {
      var parentBuildCount = 0;
      var localConsumerBuildCount = 0;

      await tester.pumpWidget(
        MaterialApp(
          home: CounterViewModelProvider(
            child: Column(
              children: [
                // Parent watcher - will rebuild on parent state change
                Builder(
                  builder: (context) {
                    parentBuildCount++;
                    final counter = context.counterViewModel;
                    return Text('Parent: ${counter.state}');
                  },
                ),
                // Local provider as sibling - should NOT rebuild on parent change
                LocalFormViewModelProvider(
                  child: Builder(
                    builder: (context) {
                      localConsumerBuildCount++;
                      final form = context.localFormViewModel;
                      return Text('Form: ${form.state.name}');
                    },
                  ),
                ),
                Builder(
                  builder: (context) {
                    return ElevatedButton(
                      onPressed: () =>
                          context.readCounterViewModel().increment(),
                      child: const Text('Inc Parent'),
                    );
                  },
                ),
              ],
            ),
          ),
        ),
      );

      expect(parentBuildCount, 1);
      expect(localConsumerBuildCount, 1);
      expect(find.text('Parent: 0'), findsOneWidget);
      expect(find.text('Form: '), findsOneWidget);

      // Increment parent counter - should rebuild parent but NOT local provider
      await tester.tap(find.text('Inc Parent'));
      await tester.pump();

      expect(parentBuildCount, 2, reason: 'Parent should rebuild');
      expect(localConsumerBuildCount, 1,
          reason: 'Local consumer should NOT rebuild when parent changes');
      expect(find.text('Parent: 1'), findsOneWidget);
      expect(find.text('Form: '), findsOneWidget);
    });

    testWidgets(
        'local provider with const child does not rebuild on parent state change',
        (tester) async {
      var parentBuildCount = 0;
      var localConsumerBuildCount = 0;

      await tester.pumpWidget(
        MaterialApp(
          home: CounterViewModelProvider(
            child: Column(
              children: [
                Builder(
                  builder: (context) {
                    parentBuildCount++;
                    final counter = context.counterViewModel;
                    return Text('Parent: ${counter.state}');
                  },
                ),
                // Local provider as sibling (not child of watching Builder)
                LocalFormViewModelProvider(
                  child: Builder(
                    builder: (context) {
                      localConsumerBuildCount++;
                      final form = context.localFormViewModel;
                      return Text('Form: ${form.state.name}');
                    },
                  ),
                ),
                Builder(
                  builder: (context) {
                    return ElevatedButton(
                      onPressed: () =>
                          context.readCounterViewModel().increment(),
                      child: const Text('Inc Parent'),
                    );
                  },
                ),
              ],
            ),
          ),
        ),
      );

      expect(parentBuildCount, 1);
      expect(localConsumerBuildCount, 1);

      // Increment parent counter
      await tester.tap(find.text('Inc Parent'));
      await tester.pump();

      expect(parentBuildCount, 2, reason: 'Parent watcher should rebuild');
      expect(localConsumerBuildCount, 1,
          reason: 'Local consumer should NOT rebuild when parent changes');
      expect(find.text('Parent: 1'), findsOneWidget);
      expect(find.text('Form: '), findsOneWidget);
    });

    testWidgets(
        'local provider rebuilds only its consumers on local state change',
        (tester) async {
      var parentBuildCount = 0;
      var localConsumerBuildCount = 0;

      await tester.pumpWidget(
        MaterialApp(
          home: CounterViewModelProvider(
            child: Column(
              children: [
                Builder(
                  builder: (context) {
                    parentBuildCount++;
                    final counter = context.counterViewModel;
                    return Text('Parent: ${counter.state}');
                  },
                ),
                LocalFormViewModelProvider(
                  child: Builder(
                    builder: (context) {
                      localConsumerBuildCount++;
                      final form = context.localFormViewModel;
                      return Column(
                        children: [
                          Text('Name: ${form.state.name}'),
                          ElevatedButton(
                            onPressed: () => context
                                .readLocalFormViewModel()
                                .updateName('John'),
                            child: const Text('Update Name'),
                          ),
                        ],
                      );
                    },
                  ),
                ),
              ],
            ),
          ),
        ),
      );

      expect(parentBuildCount, 1);
      expect(localConsumerBuildCount, 1);
      expect(find.text('Name: '), findsOneWidget);

      // Update local form state
      await tester.tap(find.text('Update Name'));
      await tester.pump();

      expect(parentBuildCount, 1,
          reason: 'Parent should NOT rebuild when local state changes');
      expect(localConsumerBuildCount, 2,
          reason: 'Local consumer should rebuild');
      expect(find.text('Name: John'), findsOneWidget);
    });

    testWidgets('multiple nested local providers are independent',
        (tester) async {
      var form1BuildCount = 0;
      var form2BuildCount = 0;

      await tester.pumpWidget(
        MaterialApp(
          home: Column(
            children: [
              LocalFormViewModelProvider(
                child: Builder(
                  builder: (context) {
                    form1BuildCount++;
                    final form = context.localFormViewModel;
                    return Column(
                      children: [
                        Text('Form1: ${form.state.name}'),
                        ElevatedButton(
                          onPressed: () => context
                              .readLocalFormViewModel()
                              .updateName('Alice'),
                          child: const Text('Update Form1'),
                        ),
                      ],
                    );
                  },
                ),
              ),
              LocalFormViewModelProvider(
                child: Builder(
                  builder: (context) {
                    form2BuildCount++;
                    final form = context.localFormViewModel;
                    return Column(
                      children: [
                        Text('Form2: ${form.state.name}'),
                        ElevatedButton(
                          onPressed: () => context
                              .readLocalFormViewModel()
                              .updateName('Bob'),
                          child: const Text('Update Form2'),
                        ),
                      ],
                    );
                  },
                ),
              ),
            ],
          ),
        ),
      );

      expect(form1BuildCount, 1);
      expect(form2BuildCount, 1);
      expect(find.text('Form1: '), findsOneWidget);
      expect(find.text('Form2: '), findsOneWidget);

      // Update form1 only
      await tester.tap(find.text('Update Form1'));
      await tester.pump();

      expect(form1BuildCount, 2, reason: 'Form1 consumer should rebuild');
      expect(form2BuildCount, 1, reason: 'Form2 consumer should NOT rebuild');
      expect(find.text('Form1: Alice'), findsOneWidget);
      expect(find.text('Form2: '), findsOneWidget);

      // Update form2 only
      await tester.tap(find.text('Update Form2'));
      await tester.pump();

      expect(form1BuildCount, 2, reason: 'Form1 consumer should still be 2');
      expect(form2BuildCount, 2, reason: 'Form2 consumer should rebuild');
      expect(find.text('Form1: Alice'), findsOneWidget);
      expect(find.text('Form2: Bob'), findsOneWidget);
    });

    testWidgets('local provider read pattern does not rebuild', (tester) async {
      var buildCount = 0;

      await tester.pumpWidget(
        MaterialApp(
          home: LocalFormViewModelProvider(
            child: Builder(
              builder: (context) {
                buildCount++;
                return ElevatedButton(
                  onPressed: () =>
                      context.readLocalFormViewModel().updateName('Test'),
                  child: const Text('Update'),
                );
              },
            ),
          ),
        ),
      );

      expect(buildCount, 1);

      await tester.tap(find.text('Update'));
      await tester.pump();

      expect(buildCount, 1, reason: 'Should NOT rebuild with read pattern');
    });

    testWidgets('local provider is disposed when removed from tree',
        (tester) async {
      var showForm = true;

      await tester.pumpWidget(
        StatefulBuilder(
          builder: (context, setState) {
            return MaterialApp(
              home: Column(
                children: [
                  if (showForm)
                    LocalFormViewModelProvider(
                      child: Builder(
                        builder: (context) {
                          final form = context.localFormViewModel;
                          return Text('Name: ${form.state.name}');
                        },
                      ),
                    ),
                  ElevatedButton(
                    onPressed: () => setState(() => showForm = false),
                    child: const Text('Remove'),
                  ),
                ],
              ),
            );
          },
        ),
      );

      expect(find.text('Name: '), findsOneWidget);

      await tester.tap(find.text('Remove'));
      await tester.pump();

      // Form should be removed (and internally disposed)
      expect(find.text('Name: '), findsNothing);
    });
  });
}
