import secrets
from datetime import datetime, timedelta

import jwt
from fastapi import APIRouter, Depends, Security
from fastapi.security import OAuth2PasswordBearer, OAuth2PasswordRequestForm, SecurityScopes
from jwt import PyJWTError

from .models import WeatherDataUser, WeatherDataUsers, Permissions
from .schemas import Token


# of course the defaults should be seeded from a configuration
ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE_MINUTES = 5

# this works fine for the single server
SECRET_KEY = secrets.token_hex(32)

auth_url = "/token"
oauth2_scheme = OAuth2PasswordBearer(
    tokenUrl=auth_url,
    scopes={
        Permissions.ReadUsers: "Read information about other users.",
        Permissions.ReadWeatherData: "Read weather data locations and history."
    }
)


_api_users = WeatherDataUsers()
get_users = _api_users.get_all


async def get_current_user(security_scopes: SecurityScopes, token: str = Depends(oauth2_scheme)) -> WeatherDataUser:
    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=[ALGORITHM])
        username: str = payload.get("sub")
        if username is None:
            raise AuthenticationError("Credentials required.")
    except PyJWTError:
        raise SessionExpired()
    user = _api_users.get(username)
    if user is None:
        raise AuthenticationError("Bad credentials.")
    scopes = payload.get("scopes", [])
    if security_scopes.scopes:
        for scope in security_scopes.scopes:
            if scope not in scopes:
                raise AuthenticationError("Permission required.", permissions=security_scopes.scope_str)
    return user


async def get_current_active_user(current_user: WeatherDataUser = Security(get_current_user)) -> WeatherDataUser:
    if current_user.disabled:
        raise DisabledUser()
    return current_user


###############################################################################
# Authentication hook
###############################################################################


auth_router = APIRouter()


@auth_router.post("/", response_model=Token, include_in_schema=False)
async def _authenticate(form_data: OAuth2PasswordRequestForm = Depends()):
    user = _api_users.get(form_data.username)
    if not (user and _api_users.verify_password(user, form_data.password)):
        raise AuthenticationError("Bad credentials.")

    if form_data.scopes:
        for scope in form_data.scopes:
            if scope not in user.permissions:
                raise AuthenticationError("Bad permissions.")

    oauth2_token = {
        "sub": user.username,
        "exp": datetime.utcnow() + timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES),
        "scopes": form_data.scopes
    }
    return Token(access_token=jwt.encode(oauth2_token, SECRET_KEY, ALGORITHM), token_type="Bearer")


###############################################################################
# Authentication errors
###############################################################################

_auth_scheme = "Bearer"


class AuthenticationError(Exception):
    def __init__(self, detail: str, permissions: str = None):
        self.detail = detail
        self.auth_scheme = f'{_auth_scheme} scope={permissions}' if permissions else _auth_scheme


class SessionExpired(AuthenticationError):
    def __init__(self):
        super().__init__("Session expired.")


class DisabledUser(AuthenticationError):
    def __init__(self):
        super().__init__("User disabled.")
