from setuptools import setup, find_packages
setup(
    name="weather",
    version="1.0",
    packages=find_packages(),
    include_package_data=True,
    install_requires=[
        'Click',
        'pytz',
        'tksheet',
        'tkcalendar',
        'requests',
        'pyYAML'
    ],
    entry_points='''
        [console_scripts]
        gui=weather.gui:run_gui
        cli=weather.cli:run_cli
    ''',
)
