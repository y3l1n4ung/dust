/// Post model for pagination demo
class Post {
  final int id;
  final String title;
  final String body;
  final String author;
  final DateTime createdAt;

  const Post({
    required this.id,
    required this.title,
    required this.body,
    required this.author,
    required this.createdAt,
  });
}

/// Generate mock posts
List<Post> generateMockPosts(int page, int pageSize) {
  final startId = page * pageSize;
  final authors = ['John Doe', 'Jane Smith', 'Bob Wilson', 'Alice Brown'];

  return List.generate(pageSize, (index) {
    final id = startId + index + 1;
    return Post(
      id: id,
      title: 'Post #$id: ${_titles[id % _titles.length]}',
      body: _bodies[id % _bodies.length],
      author: authors[id % authors.length],
      createdAt: DateTime.now().subtract(Duration(hours: id * 2)),
    );
  });
}

const _titles = [
  'Getting Started with Flutter',
  'State Management Best Practices',
  'Building Responsive UIs',
  'Working with APIs',
  'Testing Your Flutter Apps',
  'Performance Optimization Tips',
  'Custom Widgets Deep Dive',
  'Navigation Patterns',
  'Animations Made Easy',
  'Deploying to Production',
];

const _bodies = [
  'Learn the fundamentals of Flutter development and start building beautiful apps today. This comprehensive guide covers everything you need to know.',
  'Discover the best practices for managing state in your Flutter applications. From simple setState to advanced solutions like BLoC and Riverpod.',
  'Create stunning user interfaces that look great on any screen size. Learn about responsive design patterns and adaptive layouts.',
  'Connect your Flutter app to backend services and APIs. Handle authentication, data fetching, and error handling like a pro.',
  'Write reliable tests for your Flutter applications. Unit tests, widget tests, and integration tests explained with practical examples.',
];
