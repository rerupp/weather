from contextlib import contextmanager

from sqlalchemy import create_engine
from sqlalchemy.orm import Session, sessionmaker


class Database:

    def __init__(self, db_url: str = "sqlite:///./weather_data.db"):
        # add a hook for sqlite3
        self.db_url = db_url
        self.is_sqlite = db_url.startswith("sqlite")
        if self.is_sqlite:
            # make sure you have support for async operations
            self.engine = create_engine(db_url, connect_args={"check_same_thread": False})
        else:
            # add support for other db backends here
            self.engine = create_engine(db_url)
        from .models import Base
        Base.metadata.create_all(bind=self.engine)
        self.session_maker = sessionmaker(autocommit=False, autoflush=False, bind=self.engine)

    @contextmanager
    def get_session(self) -> Session:
        session = self.session_maker()
        try:
            yield session
        except Exception as err:
            session.rollback()
            print(f'Yikes... error caught: {err}')
            raise
        else:
            session.commit()
        finally:
            session.close()
