import 'package:flutter/material.dart';

import '../view_models/search_view_model.dart';

/// Search View - demonstrates debounced search with real-time results
class SearchView extends StatefulWidget {
  const SearchView({super.key});

  @override
  State<SearchView> createState() => _SearchViewState();
}

class _SearchViewState extends State<SearchView> {
  final _searchController = TextEditingController();
  final _focusNode = FocusNode();

  @override
  void dispose() {
    _searchController.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final searchViewModel = context.searchViewModel;
    final search = searchViewModel.state;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Search'),
        bottom: PreferredSize(
          preferredSize: const Size.fromHeight(60),
          child: Padding(
            padding: const EdgeInsets.fromLTRB(16, 0, 16, 12),
            child: TextField(
              controller: _searchController,
              focusNode: _focusNode,
              decoration: InputDecoration(
                hintText: 'Search articles, tutorials...',
                prefixIcon: const Icon(Icons.search),
                suffixIcon: search.query.isNotEmpty
                    ? IconButton(
                        icon: const Icon(Icons.clear),
                        onPressed: () {
                          _searchController.clear();
                          searchViewModel.clearSearch();
                        },
                      )
                    : null,
                filled: true,
                fillColor: Theme.of(context).colorScheme.surface,
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(12),
                  borderSide: BorderSide.none,
                ),
                contentPadding: const EdgeInsets.symmetric(
                  horizontal: 16,
                  vertical: 12,
                ),
              ),
              onChanged: searchViewModel.updateQuery,
              onSubmitted: (query) {
                if (query.isNotEmpty) {
                  searchViewModel.addToRecentSearches(query);
                }
              },
            ),
          ),
        ),
      ),
      body: _buildBody(context, searchViewModel, search),
    );
  }

  Widget _buildBody(
    BuildContext context,
    SearchViewModel searchViewModel,
    SearchStatus search,
  ) {
    // Show loading indicator
    if (search.isLoading) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            CircularProgressIndicator(),
            SizedBox(height: 16),
            Text('Searching...'),
          ],
        ),
      );
    }

    // Show error
    if (search.error != null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.error_outline,
              size: 64,
              color: Theme.of(context).colorScheme.error,
            ),
            const SizedBox(height: 16),
            Text(search.error!),
            const SizedBox(height: 16),
            FilledButton(
              onPressed: () => searchViewModel.updateQuery(search.query),
              child: const Text('Retry'),
            ),
          ],
        ),
      );
    }

    // Show results
    if (search.showResults) {
      if (search.results.isEmpty) {
        return Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(
                Icons.search_off,
                size: 64,
                color: Theme.of(context).colorScheme.outline,
              ),
              const SizedBox(height: 16),
              Text(
                'No results for "${search.query}"',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 8),
              Text(
                'Try different keywords',
                style: TextStyle(
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
              ),
            ],
          ),
        );
      }

      return ListView.builder(
        padding: const EdgeInsets.all(16),
        itemCount: search.results.length,
        itemBuilder: (context, index) {
          final result = search.results[index];
          return Card(
            margin: const EdgeInsets.only(bottom: 12),
            child: ListTile(
              leading: CircleAvatar(
                backgroundColor:
                    _getCategoryColor(result.category).withValues(alpha: 0.2),
                child: Icon(
                  _getCategoryIcon(result.category),
                  color: _getCategoryColor(result.category),
                  size: 20,
                ),
              ),
              title: Text(
                result.title,
                style: const TextStyle(fontWeight: FontWeight.bold),
              ),
              subtitle: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const SizedBox(height: 4),
                  Text(result.description),
                  const SizedBox(height: 4),
                  Container(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 8,
                      vertical: 2,
                    ),
                    decoration: BoxDecoration(
                      color: _getCategoryColor(result.category)
                          .withValues(alpha: 0.1),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(
                      result.category,
                      style: TextStyle(
                        fontSize: 12,
                        color: _getCategoryColor(result.category),
                      ),
                    ),
                  ),
                ],
              ),
              isThreeLine: true,
              onTap: () {
                searchViewModel.addToRecentSearches(search.query);
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(content: Text('Opening: ${result.title}')),
                );
              },
            ),
          );
        },
      );
    }

    // Show recent searches or empty state
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        if (search.recentSearches.isNotEmpty) ...[
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                'Recent Searches',
                style: Theme.of(context).textTheme.titleMedium?.copyWith(
                      fontWeight: FontWeight.bold,
                    ),
              ),
              TextButton(
                onPressed: searchViewModel.clearRecentSearches,
                child: const Text('Clear'),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: search.recentSearches.map((query) {
              return ActionChip(
                avatar: const Icon(Icons.history, size: 18),
                label: Text(query),
                onPressed: () {
                  _searchController.text = query;
                  searchViewModel.updateQuery(query);
                },
              );
            }).toList(),
          ),
          const SizedBox(height: 24),
        ],
        Text(
          'Popular Topics',
          style: Theme.of(context).textTheme.titleMedium?.copyWith(
                fontWeight: FontWeight.bold,
              ),
        ),
        const SizedBox(height: 12),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            'Flutter',
            'Dart',
            'State Management',
            'API',
            'DevOps',
            'Database',
          ].map((topic) {
            return ActionChip(
              label: Text(topic),
              onPressed: () {
                _searchController.text = topic;
                searchViewModel.updateQuery(topic);
              },
            );
          }).toList(),
        ),
        const SizedBox(height: 32),
        Center(
          child: Column(
            children: [
              Icon(
                Icons.search,
                size: 64,
                color: Theme.of(context)
                    .colorScheme
                    .outline
                    .withValues(alpha: 0.5),
              ),
              const SizedBox(height: 16),
              Text(
                'Search for articles and tutorials',
                style: TextStyle(
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }

  Color _getCategoryColor(String category) {
    switch (category) {
      case 'Development':
        return Colors.blue;
      case 'Documentation':
        return Colors.green;
      case 'Architecture':
        return Colors.orange;
      case 'Database':
        return Colors.purple;
      case 'DevOps':
        return Colors.teal;
      default:
        return Colors.grey;
    }
  }

  IconData _getCategoryIcon(String category) {
    switch (category) {
      case 'Development':
        return Icons.code;
      case 'Documentation':
        return Icons.description;
      case 'Architecture':
        return Icons.architecture;
      case 'Database':
        return Icons.storage;
      case 'DevOps':
        return Icons.cloud;
      default:
        return Icons.article;
    }
  }
}
