import 'package:flutter/material.dart';
import 'package:state_management_prototype/features/posts/view_models/post_detail_view_model.dart';

class PostDetailPage extends StatelessWidget {
  const PostDetailPage({super.key});

  @override
  Widget build(BuildContext context) {
    final viewModel = context.watchPostDetailViewModel();
    final state = viewModel.value;
    final postId = context.readPostDetailViewModel().postId;

    return Scaffold(
      appBar: AppBar(title: Text('Post #$postId')),
      body: _buildBody(context, state),
    );
  }

  Widget _buildBody(BuildContext context, PostDetailState state) {
    if (state.isLoading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (state.errorMessage != null) {
      return Center(child: Text(state.errorMessage!));
    }

    final post = state.post;
    if (post == null) {
      return const Center(child: Text('Post not found'));
    }

    return Padding(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            post.title,
            style: Theme.of(
              context,
            ).textTheme.headlineSmall?.copyWith(fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 16),
          Text(post.body, style: Theme.of(context).textTheme.bodyLarge),
        ],
      ),
    );
  }
}
