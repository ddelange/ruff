try:
    pass
except ValueError:
    pass
except Exception as e:
    raise e
finally:
    pass


try:
    pass
except BaseException as e:
    raise e
except TypeError:
    pass
else:
    pass


try:
    pass
except Exception as e:
    raise e
except BaseException:
    pass


try:
    pass
except Exception:
    pass
finally:
    try:
        pass
    except BaseException as e:
        raise e


try:
    pass
except Exception as e:
    try:
        raise e
    except BaseException:
        pass


try:
    try:
        pass
    except BaseException as e:
        raise e
except Exception:
    pass


try:
    pass
except Exception as e:
    raise bad
except BaseException:
    pass

import logging

try:
    pass
except Exception:
    logging.error("...")


try:
    pass
except Exception:
    logging.error("...", exc_info=False)


try:
    pass
except Exception:
    logging.error("...", exc_info=None)


try:
    pass
except Exception:
    logging.exception("...")


try:
    pass
except Exception:
    logging.error("...", exc_info=True)


from logging import critical, error, exception

try:
    pass
except Exception:
    error("...")


try:
    pass
except Exception:
    error("...", exc_info=False)


try:
    pass
except Exception:
    error("...", exc_info=None)


try:
    pass
except Exception:
    critical("...")


try:
    pass
except Exception:
    critical("...", exc_info=False)


try:
    pass
except Exception:
    critical("...", exc_info=None)

try:
    pass
except Exception:
    exception("...")


try:
    pass
except Exception:
    error("...", exc_info=True)


try:
    pass
except Exception:
    critical("...", exc_info=True)


try:
    ...
except Exception as e:
    raise ValueError from e


try:
    pass
except Exception:
    if True:
        exception("An error occurred")
    else:
        exception("An error occurred")
