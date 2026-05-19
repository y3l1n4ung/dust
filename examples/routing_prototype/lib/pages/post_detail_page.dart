import 'package:flutter/material.dart' hide Route;

import '../route.dart';
import 'page_scaffold.dart';

@Route(
  '/posts/:id',
  name: 'postDetail',
  transition: CupertinoPageTransitionsBuilder(),
)
class PostDetailPage extends StatelessWidget {
  const PostDetailPage({required this.id, super.key});

  final int id;

  @override
  Widget build(BuildContext context) {
    return PageScaffold(
      title: 'Post #$id',
      subtitle: 'A push route with a typed integer path parameter.',
      badges: const [
        StatusPill('push()', icon: Icons.call_made_outlined),
        StatusPill('int path param', icon: Icons.tag_outlined),
      ],
      body: InfoCard(
        title: 'Route data class',
        icon: Icons.article_outlined,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('Path parameter was derived from constructor: int id.'),
            const SizedBox(height: 12),
            CodePill('/posts/$id'),
            const SizedBox(height: 18),
            OutlinedButton.icon(
              onPressed: context.routes.pop,
              icon: const Icon(Icons.arrow_back),
              label: const Text('Pop'),
            ),
          ],
        ),
      ),
    );
  }
}
