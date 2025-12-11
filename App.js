import React, { useState, useEffect } from 'react';
import {
  View,
  TouchableOpacity,
  Text,
  Platform,
  PermissionsAndroid,
  Alert,
} from 'react-native';
import MapboxGL from '@rnmapbox/maps';
import Geolocation from '@react-native-community/geolocation';
import { styles } from './styles';

MapboxGL.setAccessToken('pk.eyJ1Ijoib2xlZ3RvdGVtIiwiYSI6ImNtaHV6Nzl4dDA0azQybHNneWlsdThvd3IifQ.2Awquq3TcmtSdir4VhZkeg');

const FIXED_COORDINATES = {
  latitude: 49.8180161,
  longitude: 24.022562,
};

const SERVER_URL = 'http://10.10.229.67:8080/coordinates';

const App = () => {
  const [userLocation, setUserLocation] = useState(null);
  const [targetCoordinate, setTargetCoordinate] = useState(FIXED_COORDINATES);
  const [hasLocationPermission, setHasLocationPermission] = useState(false);
  const [locationError, setLocationError] = useState(null);
  const [serverStatus, setServerStatus] = useState('Not connected');
  const [lastDataReceived, setLastDataReceived] = useState(null);

  useEffect(() => {
    requestLocationPermission();
    
    const interval = setInterval(() => {
      fetchCoordinates();
    }, 5000);

    fetchCoordinates();

    return () => clearInterval(interval);
  }, []);

  const fetchCoordinates = async () => {
    try {
      const response = await fetch(SERVER_URL, {
        method: 'GET',
        headers: {
          'Accept': 'application/json',
        },
        timeout: 3000,
      });

      if (!response.ok) {
        throw new Error('Server returned ' + response.status);
      }

      const data = await response.json();
      
      if (data.latitude && data.longitude) {
        setTargetCoordinate({
          latitude: data.latitude,
          longitude: data.longitude,
        });
        
        setLastDataReceived(data.timestamp || new Date().toLocaleTimeString());
        setServerStatus(`Connected - Last: ${data.timestamp || new Date().toLocaleTimeString()}`);
      }
    } catch (error) {
      console.log('Fetch error:', error.message);
      setServerStatus('Server offline: ' + error.message);
    }
  };

  const requestLocationPermission = async () => {
    if (Platform.OS === 'android') {
      try {
        const granted = await PermissionsAndroid.request(
          PermissionsAndroid.PERMISSIONS.ACCESS_FINE_LOCATION,
          {
            title: 'Location Permission',
            message: 'This app needs access to your location.',
            buttonPositive: 'OK',
          }
        );
        
        if (granted === PermissionsAndroid.RESULTS.GRANTED) {
          setHasLocationPermission(true);
          startLocationTracking();
        }
      } catch (err) {
        console.warn('Permission error:', err);
        setLocationError(err.message);
      }
    }
  };

  const startLocationTracking = () => {
    Geolocation.getCurrentPosition(
      (position) => {
        setUserLocation({
          latitude: position.coords.latitude,
          longitude: position.coords.longitude,
        });
        setLocationError(null);
      },
      (error) => console.log('No cached location'),
      { enableHighAccuracy: false, timeout: 5000, maximumAge: 300000 }
    );

    setTimeout(() => {
      Geolocation.getCurrentPosition(
        (position) => {
          setUserLocation({
            latitude: position.coords.latitude,
            longitude: position.coords.longitude,
          });
          setLocationError(null);
        },
        (error) => {
          if (!userLocation) {
            setLocationError('Using network location');
          }
        },
        { enableHighAccuracy: true, timeout: 30000, maximumAge: 0 }
      );
    }, 1000);

    Geolocation.watchPosition(
      (position) => {
        setUserLocation({
          latitude: position.coords.latitude,
          longitude: position.coords.longitude,
        });
        setLocationError(null);
      },
      (error) => console.log('Watch error:', error),
      { enableHighAccuracy: false, distanceFilter: 10 }
    );
  };

  const handleGetCoordinates = () => {
    fetchCoordinates();
  };

  const downloadUkraineMap = async () => {
    try {
      await MapboxGL.offlineManager.createPack(
        {
          name: 'ukraine-map',
          styleURL: MapboxGL.StyleURL.Street,
          bounds: [[22.13, 44.39], [40.23, 52.38]],
          minZoom: 5,
          maxZoom: 14,
        },
        (offlineRegion, status) => console.log(`Progress: ${status.percentage}%`),
        (offlineRegion, error) => console.log('Download error:', error)
      );
      Alert.alert('Success', 'Ukraine map downloaded!');
    } catch (error) {
      Alert.alert('Error', 'Failed to download map');
    }
  };

  return (
    <View style={styles.container}>
      <MapboxGL.MapView style={styles.map} styleURL={MapboxGL.StyleURL.Street}>
        <MapboxGL.Camera
          zoomLevel={16}
          centerCoordinate={[targetCoordinate.longitude, targetCoordinate.latitude]}
          animationDuration={1000}
        />

        <MapboxGL.PointAnnotation
          id="target"
          coordinate={[targetCoordinate.longitude, targetCoordinate.latitude]}
        >
          <View style={styles.targetMarker}>
            <View style={styles.targetDot} />
            <Text style={styles.targetLabel}>Target</Text>
          </View>
        </MapboxGL.PointAnnotation>

        {hasLocationPermission && userLocation && (
          <MapboxGL.PointAnnotation
            id="userLocation"
            coordinate={[userLocation.longitude, userLocation.latitude]}
          >
            <View style={styles.userMarker}>
              <View style={styles.userDot} />
              <Text style={styles.userLabel}>You</Text>
            </View>
          </MapboxGL.PointAnnotation>
        )}
      </MapboxGL.MapView>

      <TouchableOpacity style={styles.button} onPress={handleGetCoordinates}>
        <Text style={styles.buttonText}>Get Coordinates</Text>
      </TouchableOpacity>

      <TouchableOpacity style={styles.downloadButton} onPress={downloadUkraineMap}>
        <Text style={styles.buttonText}>Download Ukraine Map</Text>
      </TouchableOpacity>

      <View style={styles.infoBox}>
        <Text style={[styles.infoText, {fontWeight: 'bold', color: serverStatus.includes('Connected') ? '#34C759' : '#FF3B30'}]}>
          Server: {serverStatus}
        </Text>
        <Text style={styles.infoText}>
          Target: {targetCoordinate.latitude.toFixed(7)}, {targetCoordinate.longitude.toFixed(7)}
        </Text>
        {userLocation && (
          <Text style={styles.infoText}>
            You: {userLocation.latitude.toFixed(7)}, {userLocation.longitude.toFixed(7)}
          </Text>
        )}
      </View>
    </View>
  );
};

export default App;