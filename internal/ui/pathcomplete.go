package ui

import (
	"os"
	"path/filepath"
	"sort"
	"strings"
)

func expandTilde(path string) string {
	if strings.HasPrefix(path, "~/") {
		home, err := os.UserHomeDir()
		if err == nil {
			return filepath.Join(home, path[2:])
		}
	}
	return path
}

func findPathCompletions(input string) []string {
	if input == "" {
		return []string{}
	}

	expandedPath := expandTilde(input)

	dir := filepath.Dir(expandedPath)
	base := filepath.Base(expandedPath)

	if dir == "." && !strings.Contains(input, "/") {
		dir, _ = os.Getwd()
	}

	entries, err := os.ReadDir(dir)
	if err != nil {
		return []string{}
	}

	var matches []string
	for _, entry := range entries {
		name := entry.Name()

		if strings.HasPrefix(strings.ToLower(name), strings.ToLower(base)) {
			fullPath := filepath.Join(dir, name)

			if strings.HasPrefix(input, "~/") {
				home, err := os.UserHomeDir()
				if err == nil && strings.HasPrefix(fullPath, home) {
					fullPath = "~" + strings.TrimPrefix(fullPath, home)
				}
			}

			if entry.IsDir() {
				fullPath += "/"
			}

			matches = append(matches, fullPath)
		}
	}

	sort.Strings(matches)
	return matches
}

func getNextCompletion(input string, currentCompletions []string, currentIndex int) (string, []string, int) {
	if input == "" {
		return "", []string{}, -1
	}

	completions := findPathCompletions(input)

	if len(completions) == 0 {
		return input, []string{}, -1
	}

	if len(completions) == 1 {
		return completions[0], completions, 0
	}

	if !stringSlicesEqual(completions, currentCompletions) {
		return completions[0], completions, 0
	}

	nextIndex := (currentIndex + 1) % len(completions)
	return completions[nextIndex], completions, nextIndex
}

func getCommonPrefix(input string) string {
	completions := findPathCompletions(input)

	if len(completions) == 0 {
		return input
	}

	if len(completions) == 1 {
		return completions[0]
	}

	prefix := completions[0]
	for _, completion := range completions[1:] {
		prefix = commonPrefix(prefix, completion)
	}

	if len(prefix) > len(input) {
		return prefix
	}

	return input
}

func commonPrefix(a, b string) string {
	minLen := len(a)
	if len(b) < minLen {
		minLen = len(b)
	}

	for i := 0; i < minLen; i++ {
		if a[i] != b[i] {
			return a[:i]
		}
	}

	return a[:minLen]
}

func stringSlicesEqual(a, b []string) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}
