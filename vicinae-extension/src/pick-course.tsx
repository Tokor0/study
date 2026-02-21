import { List, ActionPanel, Action, getPreferenceValues, open } from "@vicinae/api";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { homedir } from "node:os";
import { execFile } from "node:child_process";
import { parse as parseTOML } from "smol-toml";

interface Course {
  faculty: string;
  code: string;
  name?: string;
  path: string;
}

interface CourseToml {
  course: {
    code: string;
    name?: string;
  };
}

interface Preferences {
  coursesDir?: string;
}

function expandTilde(path: string): string {
  if (path.startsWith("~/")) {
    return join(homedir(), path.slice(2));
  }
  return path;
}

function subdirs(dir: string): string[] {
  try {
    return readdirSync(dir, { withFileTypes: true })
      .filter((e) => e.isDirectory())
      .map((e) => e.name);
  } catch {
    return [];
  }
}

function resolveCoursesDir(): string {
  const prefs = getPreferenceValues<Preferences>();
  if (prefs.coursesDir) {
    return expandTilde(prefs.coursesDir);
  }

  // Try reading ~/.config/study/config.toml
  try {
    const configPath = join(homedir(), ".config", "study", "config.toml");
    const raw = readFileSync(configPath, "utf-8");
    const config = parseTOML(raw) as { courses_dir?: string };
    if (config.courses_dir) {
      return expandTilde(config.courses_dir);
    }
  } catch {
    // fall through
  }

  return join(homedir(), "courses");
}

function scanCourses(): Course[] {
  const root = resolveCoursesDir();
  return subdirs(root).flatMap((faculty) => {
    const facultyPath = join(root, faculty);
    return subdirs(facultyPath).flatMap((entry) => {
      const coursePath = join(facultyPath, entry);
      try {
        const raw = readFileSync(join(coursePath, "course.toml"), "utf-8");
        const config = parseTOML(raw) as unknown as CourseToml;
        return [
          {
            faculty,
            code: config.course.code,
            name: config.course.name,
            path: coursePath,
          },
        ];
      } catch {
        return [];
      }
    });
  });
}

function groupByFaculty(courses: Course[]): Record<string, Course[]> {
  return courses.reduce<Record<string, Course[]>>((acc, course) => {
    (acc[course.faculty] ??= []).push(course);
    return acc;
  }, {});
}

export default function PickCourse() {
  const courses = scanCourses();
  const grouped = groupByFaculty(courses);

  return (
    <List searchBarPlaceholder="Search courses...">
      {Object.entries(grouped).map(([faculty, courses]) => (
        <List.Section key={faculty} title={faculty.toUpperCase()}>
          {courses.map((course) => (
            <List.Item
              key={course.path}
              title={course.name ? `${course.code} ${course.name}` : course.code}
              actions={
                <ActionPanel>
                  <Action
                    title="Open Directory"
                    onAction={() => open(course.path)}
                  />
                  <Action
                    title="Study"
                    onAction={() =>
                      execFile("study", [course.code], (err) => {
                        if (err) console.error("study failed:", err);
                      })
                    }
                  />
                  <Action.CopyToClipboard
                    title="Copy Path"
                    content={course.path}
                  />
                </ActionPanel>
              }
            />
          ))}
        </List.Section>
      ))}
    </List>
  );
}
