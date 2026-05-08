import 'package:flutter/material.dart';

import '../models/theme.dart';
import '../view_models/theme_view_model.dart';

/// Theme settings View - demonstrates app-wide theme control
class ThemeView extends StatelessWidget {
  const ThemeView({super.key});

  @override
  Widget build(BuildContext context) {
    final theme = context.themeViewModel;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Theme Settings'),
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          // Theme Mode Section
          _SectionTitle(title: 'Theme Mode'),
          const SizedBox(height: 8),
          _ThemeModeSelector(
            currentMode: theme.state.mode,
            onModeChanged: theme.setMode,
          ),

          const SizedBox(height: 32),

          // Color Section
          _SectionTitle(title: 'Accent Color'),
          const SizedBox(height: 8),
          _ColorSelector(
            currentColor: theme.state.seedColor,
            onColorChanged: theme.setSeedColor,
          ),

          const SizedBox(height: 32),

          // Preview Section
          _SectionTitle(title: 'Preview'),
          const SizedBox(height: 8),
          _ThemePreview(),
        ],
      ),
    );
  }
}

class _SectionTitle extends StatelessWidget {
  final String title;

  const _SectionTitle({required this.title});

  @override
  Widget build(BuildContext context) {
    return Text(
      title,
      style: Theme.of(context).textTheme.titleMedium?.copyWith(
            fontWeight: FontWeight.bold,
          ),
    );
  }
}

class _ThemeModeSelector extends StatelessWidget {
  final AppThemeMode currentMode;
  final ValueChanged<AppThemeMode> onModeChanged;

  const _ThemeModeSelector({
    required this.currentMode,
    required this.onModeChanged,
  });

  @override
  Widget build(BuildContext context) {
    return SegmentedButton<AppThemeMode>(
      segments: const [
        ButtonSegment(
          value: AppThemeMode.light,
          icon: Icon(Icons.light_mode),
          label: Text('Light'),
        ),
        ButtonSegment(
          value: AppThemeMode.dark,
          icon: Icon(Icons.dark_mode),
          label: Text('Dark'),
        ),
        ButtonSegment(
          value: AppThemeMode.system,
          icon: Icon(Icons.settings_suggest),
          label: Text('System'),
        ),
      ],
      selected: {currentMode},
      onSelectionChanged: (selected) => onModeChanged(selected.first),
    );
  }
}

class _ColorSelector extends StatelessWidget {
  final Color currentColor;
  final ValueChanged<Color> onColorChanged;

  const _ColorSelector({
    required this.currentColor,
    required this.onColorChanged,
  });

  @override
  Widget build(BuildContext context) {
    return Wrap(
      spacing: 12,
      runSpacing: 12,
      children: themeColors.map((color) {
        final isSelected = currentColor == color;
        return GestureDetector(
          onTap: () => onColorChanged(color),
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 200),
            width: 48,
            height: 48,
            decoration: BoxDecoration(
              color: color,
              shape: BoxShape.circle,
              border: isSelected
                  ? Border.all(
                      color: Theme.of(context).colorScheme.onSurface,
                      width: 3,
                    )
                  : null,
              boxShadow: isSelected
                  ? [
                      BoxShadow(
                        color: color.withValues(alpha: 0.4),
                        blurRadius: 8,
                        spreadRadius: 2,
                      )
                    ]
                  : null,
            ),
            child: isSelected
                ? const Icon(Icons.check, color: Colors.white)
                : null,
          ),
        );
      }).toList(),
    );
  }
}

class _ThemePreview extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Sample Card',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            Text(
              'This is how your app will look with the current theme settings.',
              style: Theme.of(context).textTheme.bodyMedium,
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                FilledButton(
                  onPressed: () {},
                  child: const Text('Primary'),
                ),
                const SizedBox(width: 8),
                OutlinedButton(
                  onPressed: () {},
                  child: const Text('Secondary'),
                ),
                const SizedBox(width: 8),
                TextButton(
                  onPressed: () {},
                  child: const Text('Text'),
                ),
              ],
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                _ColorChip(label: 'Primary', color: colorScheme.primary),
                const SizedBox(width: 8),
                _ColorChip(label: 'Secondary', color: colorScheme.secondary),
                const SizedBox(width: 8),
                _ColorChip(label: 'Tertiary', color: colorScheme.tertiary),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

class _ColorChip extends StatelessWidget {
  final String label;
  final Color color;

  const _ColorChip({required this.label, required this.color});

  @override
  Widget build(BuildContext context) {
    return Chip(
      avatar: CircleAvatar(backgroundColor: color),
      label: Text(label),
    );
  }
}
