import os

class Config:
    SQLALCHEMY_DATABASE_URI = "sqlite:///{0}".format(os.path.join(os.path.dirname(__file__), "server.db"))
    SQLALCHEMY_TRACK_MODIFICATIONS = False
    AMQP_BROKER_URL = os.getenv("AMQP_BROKER_URL", "amqp://app:FeelTheStreets@trycht.cz")
    ENQUEUE_RETRIES = 5