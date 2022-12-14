import numpy as np
import tensorflow as tf
import matplotlib.pyplot as plt
import pandas as pd

def plot_graphs(history, metric):
  plt.plot(history.history[metric])
  plt.plot(history.history['val_'+metric], '')
  plt.xlabel("Epochs")
  plt.ylabel(metric)
  plt.legend([metric, 'val_'+metric])

# classes = ["dance","happy","friday","clapping","clap","reaction","love","cute","party","weather","dog","funny","truth","applause","cheers","dancing","applaud","cat","animals","drinking","slow clap","bye","celebrate","angry","yay","laughing","smile","excited","weekend","movie","celebration","christmas","fun","thanksgiving","reactions","saturday","girl","monday","coffee","sad","friends","laugh","sunday","tgif","flower","rain","movies","puppy","wednesday","sun","crying","hello","yes","heart","snow","thursday","nice","no","alien","morning","yolo","storm","meme","wave","wow","art","clown","tuesday","congrats","fail","summer","swimming","good","robot","blessed","hot","turkey","cartoon","cry","fall","bunny","kitten","mad","funny animals","night","tv","life","surprise","i love you","day","omg","adorable","dogs","slap","beach","food","election","joker","interesting","love you","fire","winter","cool","good day","win","america","water","great job","mood","excellent","hug","space","hi","sports","crazy","haha","tired","what","yawn"]
classes = ["positive","negative"] 

full_dataset = pd.read_csv("data.csv")
full_dataset.dropna(how="any",inplace = True)

print(full_dataset.head())

train_features = full_dataset.copy()
train_labels = train_features.pop('tag')

encoder = tf.keras.layers.TextVectorization()
encoder.adapt(train_features)

model = tf.keras.Sequential([
    tf.keras.Input(shape=(1,), dtype=tf.string),
    encoder,
    tf.keras.layers.Embedding(len(encoder.get_vocabulary()), 64, mask_zero=True),
    tf.keras.layers.Bidirectional(tf.keras.layers.LSTM(64,  return_sequences=True)),
    tf.keras.layers.Bidirectional(tf.keras.layers.LSTM(32)),
    tf.keras.layers.Dense(64, activation='relu'),
    tf.keras.layers.Dropout(0.5),
    tf.keras.layers.Dense(len(classes), activation='softmax')
])

model.compile(loss='sparse_categorical_crossentropy',optimizer='adam',metrics=['accuracy'])

model.fit(train_features, train_labels, epochs=10)

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

model.save('model')