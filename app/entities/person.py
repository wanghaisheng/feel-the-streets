import attr
from . import Entity

@attr.s(hash=True)
class Person(Entity):
    use_step_sounds = True
    STEP_LENGTH = 0.7874 # Meters
    
    def move_to_current(self):
        self.move_to(self.position)

    def step_forward(self):
        self.move_by(self.STEP_LENGTH)
    
    def step_backward(self):
        self.move_by(-self.STEP_LENGTH)
    