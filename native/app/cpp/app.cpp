#include "cpp/app.h"

#include <QApplication>
#include <QIcon>
#include <QQmlApplicationEngine>
#include <QQuickStyle>
#include <QUrl>

std::int32_t run_app()
{
	// QApplication (not QGuiApplication) so the KDE desktop style can use the Plasma theme
	static int argc = 1;
	static char arg0[] = "gmpublisher-qt";
	static char *argv[] = {arg0, nullptr};

	QApplication app(argc, argv);
	QApplication::setApplicationName(QStringLiteral("gmpublisher-qt"));
	QApplication::setOrganizationDomain(QStringLiteral("io.github.ffseestevanli_oss"));
	QApplication::setDesktopFileName(QStringLiteral("io.github.ffseestevanli_oss.gmpublisher"));
	QApplication::setWindowIcon(QIcon::fromTheme(QStringLiteral("io.github.ffseestevanli_oss.gmpublisher")));

	// Look like a native KDE app unless the user picked another Qt Quick style
	if (qEnvironmentVariableIsEmpty("QT_QUICK_CONTROLS_STYLE")) {
		QQuickStyle::setStyle(QStringLiteral("org.kde.desktop"));
	}

	QQmlApplicationEngine engine;
	QObject::connect(&engine, &QQmlApplicationEngine::objectCreationFailed, &app, []() { QCoreApplication::exit(1); }, Qt::QueuedConnection);
	engine.load(QUrl(QStringLiteral("qrc:/qt/qml/io/github/gmpublisher/qml/Main.qml")));

	return app.exec();
}
