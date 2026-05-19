import 'package:flutter/material.dart' hide Route;

import '../layout/app_shell.dart';
import '../route.dart';
import 'page_scaffold.dart';

@Route('/projects/:projectId', name: 'project', shell: AppShell)
class ProjectPage extends StatelessWidget {
  const ProjectPage({required this.projectId, this.tab, super.key});

  final int projectId;
  final String? tab;

  @override
  Widget build(BuildContext context) {
    return PageScaffold(
      title: 'Project $projectId',
      subtitle:
          'A nested shell route with a required int path parameter and optional query tab.',
      badges: const [
        StatusPill('Shell route', icon: Icons.layers_outlined),
        StatusPill('Path + query params', icon: Icons.link_outlined),
      ],
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          InfoCard(
            title: 'Current route state',
            icon: Icons.folder_open_outlined,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('Selected tab: ${tab ?? 'overview'}'),
                const SizedBox(height: 12),
                CodePill(
                  '/projects/$projectId${tab == null ? '' : '?tab=$tab'}',
                ),
              ],
            ),
          ),
          const SizedBox(height: 18),
          Wrap(
            spacing: 12,
            runSpacing: 12,
            children: [
              FilledButton.tonal(
                onPressed: () => context.routes
                    .project(projectId: projectId, tab: 'timeline')
                    .replace(),
                child: const Text('Replace with timeline tab'),
              ),
              FilledButton.icon(
                onPressed: () => context.routes
                    .projectSettings(projectId: projectId, section: 'members')
                    .go(),
                icon: const Icon(Icons.settings_outlined),
                label: const Text('Open settings child route'),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
