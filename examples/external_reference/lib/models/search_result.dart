/// Search result item
class SearchResult {
  final String id;
  final String title;
  final String description;
  final String category;

  const SearchResult({
    required this.id,
    required this.title,
    required this.description,
    required this.category,
  });
}

/// Mock database for search
final mockSearchDatabase = [
  const SearchResult(
    id: '1',
    title: 'Flutter State Management',
    description: 'Learn about different state management solutions in Flutter',
    category: 'Development',
  ),
  const SearchResult(
    id: '2',
    title: 'React Hooks Tutorial',
    description: 'Understanding useState, useEffect and custom hooks',
    category: 'Development',
  ),
  const SearchResult(
    id: '3',
    title: 'SwiftUI Basics',
    description: 'Getting started with SwiftUI for iOS development',
    category: 'Development',
  ),
  const SearchResult(
    id: '4',
    title: 'Dart Language Guide',
    description: 'Complete guide to Dart programming language',
    category: 'Documentation',
  ),
  const SearchResult(
    id: '5',
    title: 'API Design Best Practices',
    description: 'Building RESTful APIs that scale',
    category: 'Architecture',
  ),
  const SearchResult(
    id: '6',
    title: 'Database Optimization',
    description: 'Tips for optimizing SQL and NoSQL databases',
    category: 'Database',
  ),
  const SearchResult(
    id: '7',
    title: 'CI/CD Pipeline Setup',
    description: 'Automated testing and deployment workflows',
    category: 'DevOps',
  ),
  const SearchResult(
    id: '8',
    title: 'Flutter Animation Guide',
    description: 'Creating smooth animations in Flutter apps',
    category: 'Development',
  ),
  const SearchResult(
    id: '9',
    title: 'Kubernetes Fundamentals',
    description: 'Container orchestration with K8s',
    category: 'DevOps',
  ),
  const SearchResult(
    id: '10',
    title: 'GraphQL vs REST',
    description: 'Comparing API paradigms for modern apps',
    category: 'Architecture',
  ),
];
