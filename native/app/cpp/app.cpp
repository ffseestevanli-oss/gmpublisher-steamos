#include "cpp/app.h"

#include <QApplication>
#include <QDir>
#include <QFile>
#include <QIcon>
#include <QPalette>
#include <QQmlApplicationEngine>
#include <QQuickStyle>
#include <QStandardPaths>
#include <QUrl>

// The Steventechno Edition always wears its dark rainbow colours, whatever the desktop's colour scheme is.
// KDE's styles read colours through KColorScheme, which follows KDE_COLOR_SCHEME_PATH, and everything else
// reads the application palette, so both get the same colours. GMPUBLISHER_SYSTEM_COLORS=1 keeps the desktop's.
static void applyRainbowColors(QApplication &app)
{
	if (!qEnvironmentVariableIsEmpty("GMPUBLISHER_SYSTEM_COLORS")) {
		return;
	}

	// KColorScheme needs a real file, so copy the scheme out of the resources
	QFile scheme(QStringLiteral(":/qt/qml/io/github/gmpublisher/assets/SteventechnoRainbow.colors"));
	const QString dir = QStandardPaths::writableLocation(QStandardPaths::AppDataLocation);
	const QString path = dir + QStringLiteral("/SteventechnoRainbow.colors");
	if (scheme.open(QIODevice::ReadOnly) && QDir().mkpath(dir)) {
		QFile out(path);
		if (out.open(QIODevice::WriteOnly | QIODevice::Truncate) && out.write(scheme.readAll()) >= 0) {
			out.close();
			app.setProperty("KDE_COLOR_SCHEME_PATH", path);
		}
	}

	const QColor ground(15, 14, 23), sidebar(23, 21, 31), button(31, 28, 43), raised(42, 37, 64);
	const QColor text(244, 242, 255), muted(169, 164, 196), highlight(106, 92, 255);

	QPalette palette;
	palette.setColor(QPalette::Window, ground);
	palette.setColor(QPalette::WindowText, text);
	palette.setColor(QPalette::Base, ground);
	palette.setColor(QPalette::AlternateBase, sidebar);
	palette.setColor(QPalette::Text, text);
	palette.setColor(QPalette::PlaceholderText, muted);
	palette.setColor(QPalette::Button, button);
	palette.setColor(QPalette::ButtonText, text);
	palette.setColor(QPalette::BrightText, Qt::white);
	palette.setColor(QPalette::ToolTipBase, raised);
	palette.setColor(QPalette::ToolTipText, text);
	palette.setColor(QPalette::Highlight, highlight);
	palette.setColor(QPalette::HighlightedText, Qt::white);
	palette.setColor(QPalette::Link, QColor(32, 197, 255));
	palette.setColor(QPalette::LinkVisited, QColor(208, 123, 255));
	palette.setColor(QPalette::Light, raised);
	palette.setColor(QPalette::Midlight, button);
	palette.setColor(QPalette::Mid, QColor(58, 53, 80));
	palette.setColor(QPalette::Dark, QColor(7, 6, 11));
	palette.setColor(QPalette::Shadow, Qt::black);
	for (const auto role : {QPalette::WindowText, QPalette::Text, QPalette::ButtonText}) {
		palette.setColor(QPalette::Disabled, role, muted);
	}
	app.setPalette(palette);
}

std::int32_t run_app()
{
	// QApplication (not QGuiApplication) so the KDE desktop style can use the Plasma theme
	static int argc = 1;
	static char arg0[] = "gmpublisher-qt";
	static char *argv[] = {arg0, nullptr};

	QApplication app(argc, argv);
	QApplication::setApplicationName(QStringLiteral("gmpublisher-qt"));
	QApplication::setApplicationDisplayName(QStringLiteral("gmpublisher Steventechno Edition"));
	QApplication::setOrganizationDomain(QStringLiteral("io.github.ffseestevanli_oss"));
	QApplication::setDesktopFileName(QStringLiteral("io.github.ffseestevanli_oss.gmpublisher"));
	// The bundled icon, so the AppImage has one too and an older installed icon can't win
	QApplication::setWindowIcon(QIcon(QStringLiteral(":/qt/qml/io/github/gmpublisher/assets/brand/gmpublisher-icon.png")));

	// Look like a native KDE app unless the user picked another Qt Quick style
	if (qEnvironmentVariableIsEmpty("QT_QUICK_CONTROLS_STYLE")) {
		QQuickStyle::setStyle(QStringLiteral("org.kde.desktop"));
	}
	applyRainbowColors(app);

	QQmlApplicationEngine engine;
	QObject::connect(&engine, &QQmlApplicationEngine::objectCreationFailed, &app, []() { QCoreApplication::exit(1); }, Qt::QueuedConnection);
	engine.load(QUrl(QStringLiteral("qrc:/qt/qml/io/github/gmpublisher/qml/Main.qml")));

	return app.exec();
}
