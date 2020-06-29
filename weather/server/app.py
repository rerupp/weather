import click
from logging import DEBUG, INFO
from socket import gethostname

from weather import StopWatch
from weather.configuration import get_logger, init_logging

_default_hostname = gethostname()
_default_port = 8000
_init_data = True


def click_doc(arg):
    """
    What I wanted to do was make the __doc__ for run_server show the
    hostname value and not hard code it or indicate it was mined from
    socket.gethostname(). I came across this which seems to work.
    """
    import inspect

    def decorator(function):
        if type(arg) is str:
            function.__doc__ = arg
        elif inspect.isclass(arg):
            function.__doc__ = arg.__doc__
        else:
            function.__doc__ = None
        return function

    return decorator


# Through my journey of bringing up the REST services I came across the
# click package. In the cli I used argparse but here I thought the cli
# is simple enough it would be fun to try click. I like the declarative
# approach and now that I have a decoration to create dynamic docstrings
# it's worthy of consideration.
@click.command("server", context_settings=dict(help_option_names=['-h', '--help']))
@click.option('--init/--no-init', 'initialize', default=_init_data,
              help='Initialize weather data.', show_default=True)
@click.option('-p', '--port', 'port', type=int, default=_default_port,
              help='Network port.', metavar='PORT', show_default=True)
@click.option('-v', 'verbose', count=True,
              help='Level of messaging by the server (-v, -vv, etc.).')
@click.argument("host", nargs=1, required=False, default=_default_hostname)
@click_doc(f"""
    Runs the Weather Data REST services. The host parameter and port number
    control how the services are visible on the network. Weather Data, by
    default, will be initialized.
    
    HOST is the network name of the server. [default: {_default_hostname}].
""")
def run_server(host: str, port: int, initialize: bool, verbose: int):
    init_logging()
    log = get_logger(__package__)
    log.setLevel(DEBUG if verbose else INFO)

    log.info("Initializing REST Services...")
    init_stopwatch = StopWatch(label="Complete in", in_ms=False)
    from .api import weather_data_app, initialize_weather_data
    log.info(f'{init_stopwatch}.')

    if initialize:
        initialize_weather_data()

    log.info(f'Starting uvicorn')
    import uvicorn
    log_level = "trace" if verbose > 1 else "debug" if verbose else "info"
    uvicorn.run(weather_data_app, host=host, port=port, log_level=log_level, lifespan="on")
