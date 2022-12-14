import tensorflow as tf
import numpy as np

classes = ["positive","negative"] 

model = tf.keras.models.load_model('model')

sample_text = 'i hate you'
predictions = model.predict(np.array([sample_text]))
print(sample_text)
print(predictions)
best = np.argmax(predictions)
print(classes[best])

sample_text = 'i love you'
predictions = model.predict(np.array([sample_text]))
print(sample_text)
print(predictions)
best = np.argmax(predictions)
print(classes[best])

sample_text = ('The movie was cool. The animation and the graphics '
               'were out of this world. I would recommend this movie.')
predictions = model.predict(np.array([sample_text]))
print(predictions)
best = np.argmax(predictions)
print(classes[best])

sample_text = ('The movie was not good. The animation and the graphics '
               'were terrible. I would not recommend this movie.')
predictions = model.predict(np.array([sample_text]))
print(predictions)
best = np.argmax(predictions)
print(classes[best])

sample_text = ('The movie was bad. The animation and the graphics '
               'were terrible. I would hate to see this movie.')
predictions = model.predict(np.array([sample_text]))
print(predictions)
best = np.argmax(predictions)
print(classes[best])