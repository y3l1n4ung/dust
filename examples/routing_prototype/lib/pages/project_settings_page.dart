import 'package:flutter/material.dart' hide Route;
import '../route.dart';

import 'page_scaffold.dart';

@Route('/projects/:projectId/settings', name: 'projectSettings')
class ProjectSettingsPage extends StatelessWidget {
  const ProjectSettingsPage({
    required this.projectId,
    this.section = 'general',
    super.key,
  });

  final int projectId;
  final String section;

  @override
  Widget build(BuildContext context) {
    return PageScaffold(
      title: 'Project $projectId Settings',
      subtitle:
          'A child path generated from /projects/:projectId/settings. The section query param has a constructor default.',
      badges: const [
        StatusPill('Child route', icon: Icons.account_tree_outlined),
        StatusPill('Default query param', icon: Icons.rule_outlined),
      ],
      body: InfoCard(
        title: 'Settings state',
        icon: Icons.settings_outlined,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Settings section: $section'),
            const SizedBox(height: 12),
            CodePill('/projects/$projectId/settings?section=$section'),
            const SizedBox(height: 18),
            OutlinedButton.icon(
              onPressed: () =>
                  context.routes.project(projectId: projectId).go(),
              icon: const Icon(Icons.arrow_back),
              label: const Text('Back to project'),
            ),
          ],
        ),
      ),
    );
  }
}
