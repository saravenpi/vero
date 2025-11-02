package ui

import (
	"fmt"
	"path/filepath"

	"github.com/saravenpi/vero/internal/models"
)

func formatFileSize(size int64) string {
	const unit = 1024
	if size < unit {
		return fmt.Sprintf("%d B", size)
	}
	div, exp := int64(unit), 0
	for n := size / unit; n >= unit; n /= unit {
		div *= unit
		exp++
	}
	return fmt.Sprintf("%.1f %cB", float64(size)/float64(div), "KMGTPE"[exp])
}

func getFileIcon(filename string) string {
	ext := filepath.Ext(filename)
	switch ext {
	case ".pdf":
		return "📄"
	case ".jpg", ".jpeg", ".png", ".gif", ".bmp", ".svg":
		return "🖼️"
	case ".zip", ".tar", ".gz", ".rar", ".7z":
		return "📦"
	case ".doc", ".docx":
		return "📝"
	case ".xls", ".xlsx":
		return "📊"
	case ".ppt", ".pptx":
		return "📊"
	case ".txt", ".md":
		return "📃"
	case ".mp3", ".wav", ".ogg", ".flac":
		return "🎵"
	case ".mp4", ".avi", ".mkv", ".mov":
		return "🎬"
	case ".go", ".py", ".js", ".java", ".c", ".cpp", ".rs":
		return "💻"
	default:
		return "📎"
	}
}

func renderAttachmentList(attachments []models.Attachment, selectedIdx int, showSelection bool) string {
	if len(attachments) == 0 {
		return ""
	}

	var result string
	for i, att := range attachments {
		icon := getFileIcon(att.Filename)
		sizeStr := formatFileSize(att.Size)

		if showSelection && i == selectedIdx {
			result += attachmentSelectedStyle.Render(fmt.Sprintf("▶ %s %s (%s)", icon, att.Filename, sizeStr)) + "\n"
		} else {
			result += attachmentStyle.Render(fmt.Sprintf("  %s %s (%s)", icon, att.Filename, sizeStr)) + "\n"
		}
	}

	return result
}
