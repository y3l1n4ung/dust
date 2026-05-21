sealed class AppRoutePath {
  const AppRoutePath();
}

final class DashboardPath extends AppRoutePath {
  const DashboardPath();
}

final class TasksPath extends AppRoutePath {
  const TasksPath();
}

final class ProfilePath extends AppRoutePath {
  const ProfilePath();
}

final class PostDetailPath extends AppRoutePath {
  const PostDetailPath(this.id);
  final int id;
}
